use error::*;
use reqwest::Url;
use serde_ini;
use std::collections::HashMap;
use std::fs::{DirBuilder, File};
use std::io::{self, Write};
use std::path::Path;
use unity::hub::paths;
use unity::urls::{DownloadURL, IniUrl};
use unity::{Component, Version};

#[derive(Debug)]
pub struct Manifest {
    version: Version,
    base_url: DownloadURL,
    components: HashMap<Component, ComponentData>,
}

impl Manifest {
    pub fn load(version: Version) -> Result<Manifest> {
        Self::get_manifest(version)
    }

    pub fn get(&self, component: Component) -> Option<&ComponentData> {
        self.components.get(&component)
    }

    pub fn url(&self, component: Component) -> Option<Url> {
        self.components
            .get(&component)
            .and_then(|c| self.base_url.join(&c.url).ok())
    }

    fn get_manifest(version: Version) -> Result<Manifest> {
        let cache_dir = paths::cache_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Unable to fetch cache directory")
        })?;
        DirBuilder::new().recursive(true).create(&cache_dir)?;

        let version_string = version.to_string();
        let manifest_path = cache_dir.join(&format!("{}_manifest.ini", version_string));

        if !manifest_path.exists() {
            Self::download_manifest(&version, manifest_path.to_path_buf())?;
        }
        let base_url = DownloadURL::new(&version)?;
        let manifest = File::open(manifest_path)?;
        let components: HashMap<Component, ComponentData> =
            serde_ini::from_read(manifest).chain_err(|| UvmErrorKind::ManifestReadError)?;
        Ok(Manifest {
            version,
            base_url,
            components,
        })
    }

    fn download_manifest<V, P>(version: V, path: P) -> Result<()>
    where
        V: AsRef<Version>,
        P: AsRef<Path>,
    {
        let ini_url = IniUrl::new(version)?;
        let url = ini_url.into_url();
        let body = reqwest::get(url)
            .and_then(|mut response| response.text())
            .map(|s| Self::cleanup_ini_data(&s))?;
        let mut f = File::create(path)?;
        write!(f, "{}", body)?;
        Ok(())
    }

    fn cleanup_ini_data(ini_data: &str) -> String {
        ini_data
            .lines()
            .filter(|line| {
                let line = line.trim();
                line.starts_with('[') || line
                    .split('=')
                    .next()
                    .map(|sub| sub.trim())
                    .and_then(|sub| if !sub.contains(' ') { Some(()) } else { None })
                    .is_some()
            }).collect::<Vec<&str>>()
            .join("\n")
    }
}

#[derive(Deserialize, Debug)]
pub struct ComponentData {
    pub title: String,
    pub description: String,
    url: String,
    pub size: u64,
    pub md5: Option<MD5>,
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Deserialize)]
#[serde(transparent)]
pub struct MD5(#[serde(with = "hex_serde")] pub [u8; 16]);

#[cfg(any(target_os = "windows", target_os = "macos"))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;

    #[test]
    fn fetch_metedata_for_known_unity_version_does_not_fail() {
        //2018.2.5f1: 3071d1717b71
        let version = Version::f(2018, 2, 5, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }
        Manifest::load(version).unwrap();
    }

    #[test]
    fn fetch_metedata_for_unknown_unity_version_fails() {
        let version = Version::f(2030, 1, 1, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }
        assert!(Manifest::load(version).is_err());
    }

    #[test]
    fn can_retrieve_download_url_for_component() {
        let version = Version::f(2017, 3, 0, 2);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }
        let m = Manifest::load(version).unwrap();

        #[cfg(target_os = "macos")]
        let expected_url =
            "https://download.unity3d.com/download_unity/d3a5469e8c44/MacEditorInstaller/Unity.pkg";
        #[cfg(target_os = "windows")]
        let expected_url = "https://download.unity3d.com/download_unity/d3a5469e8c44/Windows64EditorInstaller/UnitySetup64.exe";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let expected_url = "";

        assert_eq!(m.url(Component::Editor).unwrap().as_str(), expected_url);
    }

    #[test]
    fn saves_meta_file_to_cache_dir() {
        let version = Version::f(2017, 4, 9, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }

        Manifest::load(version).unwrap();
        assert!(cache_file.exists());
    }

    #[test]
    fn downloads_manifest_to_local_path() {
        let tempdir = tempfile::tempdir().unwrap();
        let version = Version::f(2018, 2, 0, 2);
        let path = tempdir
            .path()
            .join(&format!("{}_manifest.ini", version.to_string()));

        Manifest::download_manifest(version, &path).unwrap();
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
