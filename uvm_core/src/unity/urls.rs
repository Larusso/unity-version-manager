use std::ops::Deref;
use std::convert::Into;
use reqwest::Url;
use result::Result;
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
        let version = version.as_ref();
        let mut url = match version.release_type() {
            VersionType::Final => Url::parse(BASE_URL),
            _ => Url::parse(BETA_BASE_URL),
        }?;

        let hash = version.version_hash().ok_or_else(|| {
            crate::error::IllegalOperationError::new(&format!(
                "No hash value for version: {} available",
                version
            ))
        })?;
        url = url.join(&format!("{}/", hash))?;
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
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn new<V: AsRef<Version>>(version: V) -> Result<IniUrl> {
        let version = version.as_ref();
        let download_url = DownloadURL::new(version)?;

        let os = if cfg!(target_os = "macos") {
            "osx"
        } else {
            "win"
        };

        let url = download_url.join(&format!("unity-{}-{}.ini", version.to_string(), os))?;
        Ok(IniUrl(url))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    pub fn new<V: AsRef<Version>>(version: V) -> Result<IniUrl> {
        unimplemented!()
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}
