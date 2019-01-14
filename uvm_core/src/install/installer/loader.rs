use install::error::{Result, UvmInstallError, UvmInstallErrorKind};
use install::InstallVariant;
use md5::{Digest, Md5};
use reqwest::header::{RANGE, USER_AGENT};
use reqwest::StatusCode;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use unity::hub::paths;
use unity::{Component, Manifest, Version, MD5};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum CheckSumResult {
    NoCheckSum,
    NoFile,
    Equal,
    NotEqual,
    Skipped,
}

pub struct Loader {
    variant: InstallVariant,
    version: Version,
    verify: bool,
}

impl Loader {
    pub fn new(variant: InstallVariant, version: Version) -> Loader {
        Loader {
            variant,
            version,
            verify: true,
        }
    }

    pub fn verify_installer(&mut self, verify: bool) -> &mut Self {
        self.verify = verify;
        self
    }

    pub fn download(&self) -> Result<PathBuf> {
        debug!(
            "download installer for variant: {} and version: {}",
            self.variant, self.version
        );
        let manifest = Manifest::load(self.version.clone()).map_err(|err| {
            UvmInstallError::with_chain(err, UvmInstallErrorKind::ManifestLoadFailed)
        })?;
        let component: Component = self.variant.clone().into();
        let component_url = manifest
            .url(component)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to fetch installer url"))?;
        let component_data = manifest.get(component).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Failed to fetch component data")
        })?;

        let installer_dir = paths::cache_dir()
            .map(|c| c.join(&format!("installer/{}", self.version)))
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
        if installer_path.exists() {
            debug!("found installer at {}", installer_path.display());
            let r = self.verify_checksum(&installer_path, component_data.md5)?;
            if CheckSumResult::Equal == r || CheckSumResult::Skipped == r {
                return Ok(installer_path);
            } else {
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
        let mut source = response;
        let _ = io::copy(&mut source, &mut dest)?;

        fs::rename(&temp_file, &installer_path)?;

        match self.verify_checksum(&installer_path, component_data.md5)? {
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
