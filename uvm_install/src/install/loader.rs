use crate::install::error::InstallerResult;
use crate::utils;
use crate::utils::lock_process;
use crate::utils::UrlUtils;
use log::*;
use reqwest::header::{RANGE, USER_AGENT};
use reqwest::{StatusCode, Url};
use ssri::Integrity;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use unity_hub::unity::hub::paths;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum CheckSumResult {
    NoCheckSum,
    NoFile,
    Equal,
    NotEqual,
    Skipped,
}

/// Progress handler for installation operations.
///
/// This trait provides callbacks for tracking progress during downloads and installations.
///
/// # Examples
///
/// For download progress:
/// ```ignore
/// handler.set_length(total_bytes);
/// handler.inc(downloaded_bytes);
/// handler.finish();
/// ```
///
/// For installation phases (extraction, installation):
/// ```ignore
/// handler.set_message("Extracting package...");
/// // ... perform extraction ...
/// handler.set_message("Installing...");
/// // ... perform installation ...
/// handler.finish();
/// ```
pub trait ProgressHandler {
    /// Mark the operation as finished.
    fn finish(&self);

    /// Increment the progress by the given delta.
    fn inc(&self, delta: u64);

    /// Set the total length/size of the operation.
    fn set_length(&self, len: u64);

    /// Set the current position.
    #[allow(dead_code)]
    fn set_position(&self, pos: u64);

    /// Set a status message for the current phase.
    ///
    /// This is useful for showing context during installation phases
    /// that don't have deterministic progress (e.g., "Extracting...", "Installing...").
    fn set_message(&self, _msg: &str) {
        // Default no-op implementation for backward compatibility
    }

    /// Create a child progress handler for a component.
    ///
    /// This allows coordinators like MultiProgress to create individual progress bars
    /// for each component being installed. Returns None if child handlers are not supported.
    fn create_child_handler(
        &self,
        _component_name: &str,
        _component_type: &str,
    ) -> Option<Box<dyn ProgressHandler>> {
        None
    }

    /// Mark a component as complete in the overall progress.
    ///
    /// For multi-component installations, this updates the overall progress counter.
    fn mark_component_complete(&self) {
        // Default no-op implementation
    }

    /// Set the total number of components to install.
    ///
    /// This allows the progress handler to properly initialize the overall progress bar
    /// once the component count is known.
    fn set_total_components(&self, _count: usize) {
        // Default no-op implementation
    }

    /// Initialize progress tracking for all components upfront.
    ///
    /// This creates persistent progress lines for each component that will be updated
    /// as they progress through their lifecycle (downloading, installing, complete/skipped).
    fn initialize_components(&self, _components: &[(String, String)]) {
        // Default no-op implementation
    }

    /// Get a pre-created progress handler for a specific component.
    ///
    /// Returns the handler that was created during initialize_components.
    fn get_component_handler(&self, _component_id: &str) -> Option<Box<dyn ProgressHandler>> {
        None
    }

    /// Begin a determinate extraction progress display with a known total size.
    ///
    /// Unlike `set_length` (which is designed for download progress), this method
    /// switches the display to extraction mode (e.g. progress bar with "Extracting..."
    /// label) before setting the total byte count. Handlers that don't support
    /// determinate extraction progress can use the default no-op.
    fn begin_extraction_progress(&self, _total_bytes: u64) {
        // Default no-op
    }
}

pub trait InstallManifest {
    fn is_editor(&self) -> bool;
    fn id(&self) -> &str;
    fn install_size(&self) -> u64;
    fn download_url(&self) -> &str;
    fn integrity(&self) -> Option<Integrity>;
    fn install_rename_from_to<P: AsRef<Path>>(&self, base_path: P) -> Option<(PathBuf, PathBuf)>;
    fn install_destination<P: AsRef<Path>>(&self, base_path: P) -> Option<PathBuf>;
}

struct DownloadProgress<'a, R, P> {
    pub inner: R,
    pub progress_handle: &'a P,
    pub start_time: Instant,
    pub bytes_downloaded: u64,
    pub last_update: Instant,
}

// impl<'a, R: Read, P: 'a + ProgressHandler + ?Sized> Read for DownloadProgress<'a, R, &P> {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         self.inner.read(buf).map(|n| {
//             self.progress_handle.inc(n as u64);
//             n
//         })
//     }
// }

impl<'a, R: Read, P: 'a + ProgressHandler + ?Sized> Read for DownloadProgress<'a, R, Box<&P>> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_handle.inc(n as u64);
            self.bytes_downloaded += n as u64;

            // Update speed message periodically (every second)
            let now = Instant::now();
            if now.duration_since(self.last_update).as_secs() >= 1 {
                let elapsed = now.duration_since(self.start_time).as_secs_f64();
                if elapsed > 0.0 {
                    let speed = self.bytes_downloaded as f64 / elapsed;
                    let speed_msg = if speed >= 1_048_576.0 {
                        format!("{:.2} MB/s", speed / 1_048_576.0)
                    } else if speed >= 1024.0 {
                        format!("{:.2} KB/s", speed / 1024.0)
                    } else {
                        format!("{:.0} B/s", speed)
                    };
                    self.progress_handle.set_message(&speed_msg);
                }
                self.last_update = now;
            }

            n
        })
    }
}

pub struct Loader<'a, M> {
    version: &'a str,
    short_revision: &'a str,
    manifest: &'a M,
    verify: bool,
    progress_handle: Option<Box<&'a dyn ProgressHandler>>,
}

impl<'a, M> Loader<'a, M>
where
    M: InstallManifest,
{
    pub fn new(version: &'a str, short_revision: &'a str, manifest: &'a M) -> Loader<'a, M> {
        Loader {
            version,
            short_revision,
            manifest,
            verify: true,
            progress_handle: None,
        }
    }

    #[allow(dead_code)]
    pub fn verify_installer(&mut self, verify: bool) {
        self.verify = verify;
    }

    #[allow(dead_code)]
    pub fn set_progress_handle(&mut self, progress_handle: &'a dyn ProgressHandler) {
        self.progress_handle = Some(Box::new(progress_handle));
    }

    pub fn download(&self) -> InstallerResult<PathBuf> {
        let manifest = &self.manifest;
        debug!(
            "download installer for component: {} and version: {}",
            self.manifest.id(),
            self.version
        );

        let module_url = Url::parse(self.manifest.download_url())?;

        // set total size in progress
        if let Some(ref p) = self.progress_handle {
            p.set_length(manifest.install_size());
        }

        let version_string = format!("{}-{}", self.version, self.short_revision);
        let installer_dir = paths::cache_dir()
            .map(|c| c.join(&format!("installer/{}", version_string)))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Unable to fetch cache installer directory",
                )
            })?;

        let temp_dir = paths::cache_dir()
            .map(|c| c.join(&format!("tmp/{}", version_string)))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Unable to fetch cache installer directory",
                )
            })?;

        let file_name = UrlUtils::get_file_name_from_url(&module_url)?;

        let temp_file_name = format!("{}.part", file_name);

        trace!("ensure installer temp dir");
        fs::DirBuilder::new().recursive(true).create(&temp_dir)?;

        trace!("ensure installer cache dir");
        fs::DirBuilder::new()
            .recursive(true)
            .create(&installer_dir)?;

        lock_process!(temp_dir.join(format!("{}.lock", file_name)));

        let installer_path = installer_dir.join(file_name);
        trace!("installer_path: {}", installer_path.display());
        if installer_path.exists() {
            debug!("found installer at {}", installer_path.display());
            let r = self.verify_checksum(&installer_path, manifest.integrity())?;
            if CheckSumResult::Equal == r
                || CheckSumResult::Skipped == r
                || CheckSumResult::NoCheckSum == r
            {
                if let Some(ref p) = self.progress_handle {
                    p.finish();
                }
                return Ok(installer_path);
            } else {
                trace!("checksum result {:?}", r);
                fs::remove_file(&installer_path)?;
            }
        }

        let temp_file = temp_dir.join(temp_file_name);

        debug!("create tempfile for installer at {}", temp_file.display());
        //check if tempfile exists and get its size
        let start_range = if temp_file.exists() {
            let metadata = fs::metadata(&temp_file)?;
            metadata.len()
        } else {
            0
        };

        // set total size in progress
        if let Some(ref p) = self.progress_handle {
            p.inc(start_range);
        }

        debug!("request installer with offset {}", start_range);

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(module_url.as_str())
            .header(USER_AGENT, "uvm")
            .header(RANGE, format!("bytes={}-", start_range))
            .send()?;
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Download failed for {} with status {}",
                    installer_path.display(),
                    status
                ),
            )
            .into());
        }

        debug!("server responds with code {}", status);
        let append = status == StatusCode::PARTIAL_CONTENT;
        debug!("server supports partial respond {}", append);

        let mut dest = fs::OpenOptions::new()
            .append(append)
            .create(true)
            .write(true)
            .open(&temp_file)?;

        let download_start = Instant::now();

        if let Some(ref p) = self.progress_handle {
            let mut source = DownloadProgress {
                progress_handle: p,
                inner: response,
                start_time: download_start,
                bytes_downloaded: start_range,
                last_update: download_start,
            };

            let _ = io::copy(&mut source, &mut dest)?;

            // Set final completion message with time taken
            let elapsed = download_start.elapsed();
            let elapsed_msg = if elapsed.as_secs() >= 60 {
                format!("{}m {}s", elapsed.as_secs() / 60, elapsed.as_secs() % 60)
            } else {
                format!("{}s", elapsed.as_secs())
            };
            p.set_message(&format!("Downloaded in {}", elapsed_msg));
            p.finish();
        } else {
            let mut source = response;
            let _ = io::copy(&mut source, &mut dest)?;
        }

        fs::rename(&temp_file, &installer_path)?;

        match self.verify_checksum(&installer_path, self.manifest.integrity())? {
            CheckSumResult::NotEqual => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Checksum verify failed for {}", installer_path.display()),
            )
            .into()),
            CheckSumResult::NoFile => {
                Err(io::Error::new(io::ErrorKind::Other, "Failed to download installer").into())
            }
            _ => Ok(installer_path),
        }
    }

    fn verify_checksum<P: AsRef<Path>>(
        &self,
        path: P,
        check_sum: Option<Integrity>,
    ) -> io::Result<CheckSumResult> {
        if !self.verify {
            debug!("skip installer checksum verification");
            return Ok(CheckSumResult::Skipped);
        }

        let path = path.as_ref();
        if path.exists() {
            debug!("installer already downloaded at {}", path.display());
            debug!("check installer checksum");

            if let Some(i) = check_sum {
                let mut installer = fs::File::open(&path)?;
                let mut installer_bytes = Vec::new();
                installer.read_to_end(&mut installer_bytes)?;
                return match i.check(&installer_bytes) {
                    Ok(_) => Ok(CheckSumResult::Equal),
                    _ => Ok(CheckSumResult::NotEqual),
                };
            } else {
                return Ok(CheckSumResult::NoCheckSum);
            }
        }
        Ok(CheckSumResult::NoFile)
    }
}
