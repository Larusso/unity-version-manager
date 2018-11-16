use reqwest::Url;
use result::Result;
use unity::version::{Version, VersionType};

const BASE_URL: &'static str = "https://download.unity3d.com/download_unity/";
const BETA_BASE_URL: &'static str = "https://beta.unity3d.com/download/";

#[derive(Debug)]
pub struct DownloadURL(Url);
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

    pub fn to_url(self) -> Url {
        self.0
    }

    pub fn join(&self, input: &str) -> Result<Url> {
        self.0.join(input).map_err(|err| err.into())
    }
}

#[derive(Debug)]
pub struct IniUrl(Url);

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

    pub fn to_url(self) -> Url {
        self.0
    }
}
