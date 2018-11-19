use md5::{Digest, Md5};
use reqwest::header::{RANGE, USER_AGENT};
use reqwest::StatusCode;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use unity::hub::paths;
use unity::{Component, Manifest, Version, MD5};
mod installer;
pub use self::installer::install_editor;
pub use self::installer::install_module;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum InstallVariant {
    Android,
    Ios,
    WebGl,
    Linux,
    Windows,
    WindowsMono,
    Editor,
}

impl fmt::Display for InstallVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InstallVariant::Android => write!(f, "android"),
            InstallVariant::Ios => write!(f, "ios"),
            InstallVariant::WebGl => write!(f, "webgl"),
            InstallVariant::Linux => write!(f, "linux"),
            InstallVariant::Windows => write!(f, "windows"),
            InstallVariant::WindowsMono => write!(f, "windows-mono"),
            _ => write!(f, "editor"),
        }
    }
}

impl From<Component> for InstallVariant {
    fn from(component: Component) -> Self {
        match component {
            Component::Android => InstallVariant::Android,
            Component::Ios => InstallVariant::Ios,
            Component::WebGl => InstallVariant::WebGl,
            Component::Linux => InstallVariant::Linux,
            Component::Windows => InstallVariant::Windows,
            Component::WindowsMono => InstallVariant::WindowsMono,
            _ => InstallVariant::Editor,
        }
    }
}

impl From<InstallVariant> for Component {
    fn from(component: InstallVariant) -> Self {
        match component {
            InstallVariant::Android => Component::Android,
            InstallVariant::Ios => Component::Ios,
            InstallVariant::WebGl => Component::WebGl,
            InstallVariant::Linux => Component::Linux,
            InstallVariant::Windows => Component::Windows,
            InstallVariant::WindowsMono => Component::WindowsMono,
            InstallVariant::Editor => Component::Editor,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
enum CheckSumResult {
    NoCheckSum,
    NoFile,
    Equal,
    NotEqual,
}

fn verify_checksum<P: AsRef<Path>>(
    path: P,
    check_sum: Option<MD5>,
) -> ::result::Result<CheckSumResult> {
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

pub fn download_installer(variant: InstallVariant, version: &Version) -> ::result::Result<PathBuf> {
    debug!(
        "download installer for variant: {} and version: {}",
        variant, version
    );
    let manifest = Manifest::load(version.to_owned())?;
    let component: Component = variant.into();
    let component_url = manifest
        .url(&component)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to fetch installer url"))?;
    let component_data = manifest
        .get(&component)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to fetch component data"))?;

    let installer_dir = paths::cache_dir()
        .map(|c| c.join(&format!("installer/{}", version)))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Unable to fetch cache installer directory",
            )
        })?;
    let file_name = component_url.as_str().rsplit('/').next().unwrap();

    let temp_file_name = format!("{}.part", file_name);
    let lock_file_name = format!("{}.lock", file_name);

    trace!("ensure installer cache dir");
    fs::DirBuilder::new()
        .recursive(true)
        .create(&installer_dir)?;

    let lock_file = installer_dir.join(lock_file_name);
    lock_process!(lock_file);

    let installer_path = installer_dir.join(file_name);
    if installer_path.exists() {
        debug!("found installer at {}", installer_path.display());
        let r = verify_checksum(&installer_path, component_data.md5)?;
        if CheckSumResult::Equal == r {
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
        ).into());
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

    match verify_checksum(&installer_path, component_data.md5)? {
        CheckSumResult::NotEqual => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Checksum verify failed for {}", installer_path.display()),
        ).into()),
        CheckSumResult::NoFile => {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to download installer").into())
        }
        _ => Ok(installer_path),
    }
}
