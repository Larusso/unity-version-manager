use crate::error::Result;
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
use unity_hub::unity::hub::paths;
use crate::utils::lock_process;
use crate::utils;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum CheckSumResult {
    NoCheckSum,
    NoFile,
    Equal,
    NotEqual,
    Skipped,
}

pub trait ProgressHandler {
    fn finish(&self);
    fn inc(&self, delta: u64);
    fn set_length(&self, len: u64);
    fn set_position(&self, pos: u64);
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

    pub fn verify_installer(&mut self, verify: bool) {
        self.verify = verify;
    }

    pub fn set_progress_handle<P: 'a + ProgressHandler>(&mut self, progress_handle: &'a P) {
        self.progress_handle = Some(Box::new(progress_handle));
    }

    pub fn download(&self) -> Result<PathBuf> {
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

        if let Some(ref p) = self.progress_handle {
            let mut source = DownloadProgress {
                progress_handle: p,
                inner: response,
            };

            let _ = io::copy(&mut source, &mut dest)?;
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
    ) -> Result<CheckSumResult> {
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
