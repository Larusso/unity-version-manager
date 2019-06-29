use error::*;
use reqwest::Url;
use std::convert::Into;
use std::ops::Deref;
use unity::version::{Version, VersionType};

const BASE_URL: &str = "https://download.unity3d.com/download_unity/";
const BETA_BASE_URL: &str = "https://beta.unity3d.com/download/";

#[derive(Debug)]
pub struct DownloadURL(Url);

impl Deref for DownloadURL {
    type Target = Url;

    fn deref(&self) -> &Url {
        &self.0
    }
}

impl Into<Url> for DownloadURL {
    fn into(self) -> Url {
        self.into_url()
    }
}

impl DownloadURL {
    pub fn new<V: AsRef<Version>>(version: V) -> Result<DownloadURL> {
        use std::error::Error;
        let version = version.as_ref();
        let mut url = match version.release_type() {
            VersionType::Final => Url::parse(BASE_URL),
            _ => Url::parse(BETA_BASE_URL),
        }
        .map_err(|err| UvmError::with_chain(err, "failed to parse download url"))?;

        let hash = version.version_hash().map_err(|err| {
            warn!("{}", err.description());
            UvmError::with_chain(
                err,
                format!("No hash value for version: {} available", version),
            )
        })?;
        url = url
            .join(&format!("{}/", hash))
            .map_err(|err| UvmError::with_chain(err, "failed to parse hash url"))?;
        Ok(DownloadURL(url))
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}

#[derive(Debug)]
pub struct IniUrl(Url);

impl Deref for IniUrl {
    type Target = Url;

    fn deref(&self) -> &Url {
        &self.0
    }
}

impl Into<Url> for IniUrl {
    fn into(self) -> Url {
        self.into_url()
    }
}

impl IniUrl {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    pub fn new<V: AsRef<Version>>(version: V) -> Result<IniUrl> {
        let version = version.as_ref();
        let download_url = DownloadURL::new(version)?;

        let os = if cfg!(target_os = "macos") {
            "osx"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "win"
        };

        let url = download_url
            .join(&format!("unity-{}-{}.ini", version.to_string(), os))
            .map_err(|err| UvmError::with_chain(err, "failed to parse ini url"))?;
        Ok(IniUrl(url))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn new<V: AsRef<Version>>(version: V) -> Result<IniUrl> {
        unimplemented!()
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}
