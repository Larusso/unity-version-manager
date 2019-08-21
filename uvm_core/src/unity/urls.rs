use crate::error::*;
use crate::platform::Platform;
use crate::unity::version::{Version, VersionType};
use reqwest::Url;
use std::convert::Into;
use std::ops::Deref;

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

pub struct IniUrlBuilder {
    platform: Platform,
}

impl IniUrlBuilder {
    pub fn new() -> Self {
        Self {
            platform: Platform::default(),
        }
    }

    pub fn platform(&mut self, platform: Platform) -> &mut Self {
        self.platform = platform;
        self
    }

    pub fn build<V: AsRef<Version>>(&self, version: V) -> Result<IniUrl> {
        let version = version.as_ref();
        let download_url = DownloadURL::new(version)?;

        let url = download_url
            .join(&format!(
                "unity-{}-{}.ini",
                version.to_string(),
                self.platform
            ))
            .map_err(|err| UvmError::with_chain(err, "failed to parse ini url"))?;
        Ok(IniUrl(url))
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
        IniUrlBuilder::new().build(version)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn new<V: AsRef<Version>>(version: V) -> Result<IniUrl> {
        unimplemented!()
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ini_url_new_uses_default_platform() {
        let v = Version::f(2017, 1, 1, 1);
        let platform = Platform::default();
        let url: IniUrl = IniUrl::new(&v).unwrap();

        assert!(url
            .to_string()
            .as_str()
            .contains(&format!("unity-{}-{}.ini", &v, platform)));
    }

    #[test]
    fn ini_url_builder_uses_default_platform() {
        let v = Version::f(2017, 1, 1, 1);
        let platform = Platform::default();
        let url: IniUrl = IniUrlBuilder::new().build(&v).unwrap();

        assert!(url
            .to_string()
            .as_str()
            .contains(&format!("unity-{}-{}.ini", &v, platform)));
    }

    fn get_test_platform() -> Platform {
        match Platform::default() {
            Platform::MacOs => Platform::Win,
            Platform::Win => Platform::Linux,
            Platform::Linux => Platform::MacOs,
        }
    }

    #[test]
    fn ini_url_builder_can_set_platform() {
        let v = Version::f(2017, 1, 1, 1);
        let platform = get_test_platform();
        let url: IniUrl = IniUrlBuilder::new().platform(platform).build(&v).unwrap();

        assert!(url
            .to_string()
            .as_str()
            .contains(&format!("unity-{}-{}.ini", &v, platform)));
    }
}
