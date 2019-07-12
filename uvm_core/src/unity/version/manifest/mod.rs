use crate::error::*;
use crate::unity::hub::paths;
use crate::unity::urls::{DownloadURL, IniUrl};
use crate::unity::{Component, Version};
use reqwest::header;
use reqwest::Url;
use serde::Deserialize;
use serde_ini;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt;
use std::fs::{DirBuilder, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;

mod de;

lazy_static! {
    static ref CLIENT: reqwest::Client = {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("uvm"));
        reqwest::Client::builder()
            .gzip(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Create http client")
    };
}

type Components = HashMap<Component, ComponentData>;

#[derive(Debug)]
pub struct Manifest<'a> {
    version: &'a Version,
    base_url: DownloadURL,
    components: Components,
}

impl<'a> Manifest<'a> {
    pub fn load(version: &'a Version) -> Result<Manifest<'a>> {
        Self::get_manifest(version)
    }

    pub fn get(&self, component: Component) -> Option<&ComponentData> {
        self.components.get(&component)
    }

    pub fn url(&self, component: Component) -> Option<&Url> {
        self.components
            .get(&component)
            .and_then(|c| c.download_url.as_ref())
    }

    pub fn size(&self, component: Component) -> Option<u64> {
        self.components
            .get(&component)
            .map(|c| if cfg![windows] { c.size * 1024 } else { c.size })
    }

    pub fn version(&self) -> &Version {
        self.version
    }

    pub fn base_url(&self) -> &DownloadURL {
        &self.base_url
    }

    pub fn iter(&self) -> Iter<'_, Component, ComponentData> {
        self.components.iter()
    }

    fn get_manifest(version: &'a Version) -> Result<Manifest<'a>> {
        let cache_dir = paths::cache_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Unable to fetch cache directory")
        })?;

        DirBuilder::new()
            .recursive(true)
            .create(&cache_dir)
            .map_err(|_err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Unable to create cache directory at {}",
                        cache_dir.display()
                    ),
                )
            })?;

        let manifest_path = cache_dir.join(&format!("{}_manifest.ini", version));

        if !manifest_path.exists() {
            Self::download_manifest(version, manifest_path.to_path_buf())
        } else {
            let base_url = DownloadURL::new(&version)?;
            let manifest = File::open(manifest_path)?;
            let components = Self::read_components(manifest, &base_url)?;
            Ok(Manifest {
                version,
                base_url,
                components,
            })
        }
    }

    fn read_components<R: Read>(input: R, base_url: &DownloadURL) -> Result<Components> {
        serde_ini::from_read(input)
            .chain_err(|| UvmErrorKind::ManifestReadError)
            .map(|mut components: Components| {
                for (_, data) in components.iter_mut() {
                    data.download_url = base_url.join(&data.url).ok()
                }
                components
            })
    }

    fn download_manifest<P>(version: &'a Version, path: P) -> Result<Manifest<'a>>
    where
        P: AsRef<Path>,
    {
        let version = version;
        let ini_url = IniUrl::new(version)?;
        let url = ini_url.into_url();
        debug!("Manifest URL {}", &url);

        let client = &CLIENT;
        let request = client.get(url).build()?;

        debug!("Manifest Request:");
        debug!("{:?}", request);
        let mut response = client.execute(request)?;

        debug!("Manifest Repsonse:");
        debug!("{:?}", response);

        if !response.status().is_success() {
            trace!("{}", response.text()?);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unable to load manifest. Status: {}", response.status()),
            )
            .into());
        }

        let body = response.text()?;
        let body = Self::cleanup_ini_data(&body);

        let base_url = DownloadURL::new(version)?;
        let components = Self::read_components(body.as_bytes(), &base_url)?;

        File::create(path.as_ref())
            .and_then(|mut f| write!(f, "{}", body))
            .unwrap_or_else(|err| {
                warn!(
                    "Error while creating the manifest cache file for {}",
                    path.as_ref().display()
                );
                warn!("{}", err);
            });

        Ok(Manifest {
            version,
            base_url,
            components,
        })
    }

    fn cleanup_ini_data(ini_data: &str) -> String {
        ini_data
            .lines()
            .filter(|line| {
                let line = line.trim();
                line.starts_with('[')
                    || line
                        .split('=')
                        .next()
                        .map(|sub| sub.trim())
                        .and_then(|sub| if !sub.contains(' ') { Some(()) } else { None })
                        .is_some()
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

impl IntoIterator for Manifest<'_> {
    type Item = (Component, ComponentData);
    type IntoIter = ::std::collections::hash_map::IntoIter<Component, ComponentData>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.into_iter()
    }
}

#[derive(Deserialize, Debug)]
pub struct ComponentData {
    pub title: String,
    pub description: String,
    pub url: String,
    #[serde(skip)]
    pub download_url: Option<Url>,
    pub size: u64,
    pub installedsize: u64,
    pub md5: Option<MD5>,
    #[serde(with = "de::bool")]
    #[serde(default = "de::bool::default")]
    pub hidden: bool,
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl fmt::Display for ComponentData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.title, self.url)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct MD5(#[serde(with = "hex_serde")] pub [u8; 16]);

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;

    #[cfg(target_os = "macos")]
    #[test]
    fn fetch_metadata_for_known_unity_version_does_not_fail() {
        let version = Version::f(2019, 1, 6, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }
        Manifest::load(&version).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn fetch_metedata_for_unknown_unity_version_fails() {
        let version = Version::f(2030, 1, 1, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }
        assert!(Manifest::load(&version).is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn can_retrieve_download_url_for_component() {
        let version = Version::f(2019, 1, 6, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .expect("to retrieve cache file path");
        if cache_file.exists() {
            fs::remove_file(&cache_file).expect("delete cache file");
        }
        let m = Manifest::load(&version).expect("manifest can be loaded");

        #[cfg(target_os = "macos")]
        let expected_url =
            "https://download.unity3d.com/download_unity/f2970305fe1c/MacEditorInstaller/Unity.pkg";
        #[cfg(target_os = "windows")]
        let expected_url = "https://download.unity3d.com/download_unity/f2970305fe1c/Windows64EditorInstaller/UnitySetup64.exe";
        #[cfg(target_os = "linux")]
        let expected_url = "https://download.unity3d.com/download_unity/f2970305fe1c/LinuxEditorInstaller/Unity.tar.xz";
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let expected_url = "";

        assert_eq!(
            m.url(Component::Editor)
                .expect("fetch component url")
                .as_str(),
            expected_url
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn saves_meta_file_to_cache_dir() {
        let version = Version::f(2019, 1, 7, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }

        Manifest::load(&version).unwrap();
        assert!(cache_file.exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn downloads_manifest_to_local_path() {
        let tempdir = tempfile::tempdir().unwrap();
        let version = Version::f(2018, 2, 0, 2);
        let path = tempdir
            .path()
            .join(&format!("{}_manifest.ini", version.to_string()));

        Manifest::download_manifest(&version, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn cleanup_ini_data_removes_junk_lines() {
        let test_ini = r#"[Section1]
key=value
[Section2]
line which is not a section
key = value
line which is not a section or key=value
line which is not a section or key = value
key = "value with equals = ssjdd"
key2=value2"#;

        let expected_ini = r#"[Section1]
key=value
[Section2]
key = value
key = "value with equals = ssjdd"
key2=value2"#;
        assert_eq!(Manifest::cleanup_ini_data(test_ini), expected_ini);
    }
}
