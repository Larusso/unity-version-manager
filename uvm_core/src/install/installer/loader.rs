use crate::install::error::Result;
use crate::progress::ProgressHandler;
use crate::unity::hub::paths;
use crate::unity::{v2::Manifest, Component, MD5};
use log::*;
use md5::{Digest, Md5};
use reqwest::header::{RANGE, USER_AGENT};
use reqwest::StatusCode;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum CheckSumResult {
    NoCheckSum,
    NoFile,
    Equal,
    NotEqual,
    Skipped,
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

pub struct Loader<'a> {
    component: Component,
    manifest: &'a Manifest<'a>,
    verify: bool,
    progress_handle: Option<Box<&'a dyn ProgressHandler>>,
}

impl<'a> Loader<'a> {
    pub fn new<C>(component: C, manifest: &'a Manifest<'a>) -> Loader<'a>
    where
        C: Into<Component>,
    {
        let component = component.into();

        Loader {
            component,
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
            self.component,
            manifest.version()
        );

        let component: Component = self.component;
        let component_url = manifest
            .url(component)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to fetch installer url"))?;
        let component_data = manifest.get(&component).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Failed to fetch component data")
        })?;

        // set total size in progress
        if let Some(ref p) = self.progress_handle {
            if let Some(size) = manifest.size(component) {
                p.set_length(size);
            }
        }

        let installer_dir = paths::cache_dir()
            .map(|c| c.join(&format!("installer/{}", manifest.version())))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Unable to fetch cache installer directory",
                )
            })?;
        let file_name = component_url.as_str().rsplit('/').next().unwrap();

        let temp_file_name = format!("{}.part", file_name);

        trace!("ensure installer cache dir");
        fs::DirBuilder::new()
            .recursive(true)
            .create(&installer_dir)?;

        lock_process!(installer_dir.join(format!("{}.lock", file_name)));

        let installer_path = installer_dir.join(file_name);
        trace!("installer_path: {}", installer_path.display());
        if installer_path.exists() {
            debug!("found installer at {}", installer_path.display());
            let r = self.verify_checksum(&installer_path, component_data.checksum)?;
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

        let temp_file = installer_dir.join(temp_file_name);

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

        let client = reqwest::Client::new();
        let response = client
            .get(component_url.as_str())
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

        match self.verify_checksum(&installer_path, component_data.checksum)? {
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
        check_sum: Option<MD5>,
    ) -> Result<CheckSumResult> {
        if !self.verify {
            debug!("skip intaller checksum verification");
            return Ok(CheckSumResult::Skipped);
        }

        let path = path.as_ref();
        if path.exists() {
            debug!("installer already downloaded at {}", path.display());
            debug!("check installer checksum");
            if let Some(md5) = check_sum {
                let mut hasher = Md5::new();
                let mut installer = fs::File::open(&path)?;
                io::copy(&mut installer, &mut hasher)?;
                let hash = hasher.result();
                if hash[..] == md5.0 {
                    debug!("checksum check success.");
                    return Ok(CheckSumResult::Equal);
                } else {
                    warn!("checksum miss match.");
                    return Ok(CheckSumResult::NotEqual);
                }
            } else {
                return Ok(CheckSumResult::NoCheckSum);
            }
        }
        Ok(CheckSumResult::NoFile)
    }
}
