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

    pub fn read_manifest_version_from_path<P: AsRef<Path>>(manifest_path: P) -> Result<Version> {
        let f = File::open(manifest_path)?;
        Self::read_manifest_version(f)
    }

    pub fn read_manifest_version<R: Read>(mut reader: R) -> Result<Version> {
        use std::str::FromStr;
        let mut manifest_buffer = String::new();
        reader.read_to_string(&mut manifest_buffer)?;
        Version::from_str(&manifest_buffer).chain_err(|| "can't read version from manifest body")
    }

    pub fn from_reader<R: Read>(version: &'a Version, manifest: R) -> Result<Manifest<'a>> {
        let base_url = DownloadURL::new(&version)?;
        let components = Self::read_components(manifest, &base_url, version)?;
        Ok(Manifest {
            version,
            base_url,
            components,
        })
    }

    pub fn new<P:AsRef<Path>>(version: &'a Version, manifest_path: P) -> Result<Manifest<'a>> {
        let manifest = File::open(manifest_path)?;
        Self::from_reader(version, manifest)
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
            Manifest::new(version, manifest_path)
        }
    }

    fn read_components<R: Read, V:AsRef<Version>>(input: R, base_url: &DownloadURL, version:V) -> Result<Components> {
        let version = version.as_ref();
        serde_ini::from_read(input)
            .chain_err(|| UvmErrorKind::ManifestReadError)
            .map(|mut components: Components| {
                for (component, data) in components.iter_mut() {
                    data.download_url = component.download_url(base_url, &data.url, version)
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
        let components = Self::read_components(body.as_bytes(), &base_url, version)?;

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

use std::iter::Zip;
use std::iter::Cycle;
use std::vec::IntoIter as VecIter;
use std::collections::hash_map::IntoIter as HashIter;

pub type ManifestIteratorItem<'a> = ((Component, ComponentData), &'a Version);

impl<'a> IntoIterator for Manifest<'a> {
    type Item = ManifestIteratorItem<'a>;
    type IntoIter = Zip<HashIter<Component, ComponentData>,Cycle<VecIter<&'a Version>>>;

    fn into_iter(self) -> Self::IntoIter {
        let version_iterator = vec![self.version].into_iter().cycle();
        self.components.into_iter().zip(version_iterator)
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="eulaurl1")]
    pub eula_url_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="eulalabel1")]
    pub eula_label_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename="eulamessage")]
    pub eula_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<Component>,
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
    #[cfg(target_os = "macos")]
    use std::fs;
    #[cfg(target_os = "macos")]
    use tempfile;
    use stringreader::StringReader;

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
            "https://download.unity3d.com/download_unity/f2970305fe1c/MacEditorInstaller/Unity-2019.1.6f1.pkg";
        #[cfg(target_os = "windows")]
        let expected_url = "https://download.unity3d.com/download_unity/f2970305fe1c/Windows64EditorInstaller/UnitySetup64-2019.1.6f1.exe";
        #[cfg(target_os = "linux")]
        let expected_url = "https://download.unity3d.com/download_unity/f2970305fe1c/LinuxEditorInstaller/Unity.tar-2019.1.6f1.xz";
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
    fn can_read_version_from_manifest_body() {
        let test_ini = r#"[Section1]
key=value
[Section2]
line which is not a section
version = 2019.1.0f1
line which is not a section or key=value
line which is not a section or key = value
key = "value with equals = ssjdd"
key2=value2"#;
        let test_ini = StringReader::new(test_ini);
        let version = Manifest::read_manifest_version(test_ini).expect("a version from manifest");
        assert_eq!(version, Version::f(2019,1,0,1));
    }

    #[test]
    fn can_read_manifest_from_reader() {
        let test_ini = r#"[Unity]
title=Unity 2018.4.0f1
description=Unity Editor
url=MacEditorInstaller/Unity.pkg
install=true
mandatory=false
size=989685761
installedsize=2576390000
version=2018.4.0f1
md5=822c52aa75af582318c5d0ef94137f40
[Mono]
title=Mono for Visual Studio for Mac
description=Required for Visual Studio for Mac
url=https://go.microsoft.com/fwlink/?linkid=857641
install=false
mandatory=false
size=500000000
installedsize=1524000000
sync=VisualStudio
hidden=true
extension=pkg
[VisualStudio]
title=Visual Studio for Mac
description=Script IDE with Unity integration and debugging support. Also installs Mono, required for Visual Studio for Mac to run
url=https://go.microsoft.com/fwlink/?linkid=873167
install=true
mandatory=false
size=820000000
installedsize=2304000000
eulamessage=Please review and accept the license terms before downloading and installing Visual Studio for Mac and Mono.
eulalabel1=Visual Studio for Mac License Terms
eulaurl1=https://www.visualstudio.com/license-terms/visual-studio-mac-eula/
eulalabel2=Mono License Terms
eulaurl2=http://www.mono-project.com/docs/faq/licensing/
appidentifier=com.microsoft.visual-studio
extension=dmg
[Android]
title=Android Build Support
description=Allows building your Unity projects for the Android platform
url=MacEditorTargetInstaller/UnitySetup-Android-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=622757914
installedsize=1885331000
requires_unity=true
md5=dba5dab1ded52b75a400171579dd3940
[iOS]
title=iOS Build Support
description=Allows building your Unity projects for the iOS platform
url=MacEditorTargetInstaller/UnitySetup-iOS-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=1115793461
installedsize=2847287000
requires_unity=true
md5=0d7a1a05d61d73d07205b74c73da7741
[AppleTV]
title=tvOS Build Support
description=Allows building your Unity projects for the AppleTV platform
url=MacEditorTargetInstaller/UnitySetup-AppleTV-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=379578397
installedsize=1016195000
requires_unity=true
md5=7f429c1fc4a03d7bdef8fb9b73b393c5
[Linux]
title=Linux Build Support
description=Allows building your Unity projects for the Linux platform
url=MacEditorTargetInstaller/UnitySetup-Linux-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=276383772
installedsize=848256000
requires_unity=true
md5=02c0cd88959f7d28d9edb46d717a5efd
[Mac-IL2CPP]
title=Mac Build Support (IL2CPP)
description=Allows building your Unity projects for the Mac-IL2CPP platform
url=MacEditorTargetInstaller/UnitySetup-Mac-IL2CPP-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=86886432
installedsize=310706000
requires_unity=true
md5=0b147e6349c798549f5a9742e9e6ac33
[Vuforia-AR]
title=Vuforia Augmented Reality Support
description=Allows building your Unity projects for the Vuforia-AR platform
url=MacEditorTargetInstaller/UnitySetup-Vuforia-AR-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=149641238
installedsize=277990000
requires_unity=true
md5=b6d356215ebce9f3fb63984391755eec
[WebGL]
title=WebGL Build Support
description=Allows building your Unity projects for the WebGL platform
url=MacEditorTargetInstaller/UnitySetup-WebGL-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=324638752
installedsize=882122000
requires_unity=true
md5=a5d8a2cc47081c50e238afb6e62a16ce
[Windows-Mono]
title=Windows Build Support (Mono)
description=Allows building your Unity projects for the Windows-Mono platform
url=MacEditorTargetInstaller/UnitySetup-Windows-Mono-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=104425498
installedsize=346767000
requires_unity=true
md5=5fccd81dbd8570dbddcd8d4cfcf7fbf1
[Facebook-Games]
title=Facebook Gameroom Build Support
description=Allows building your Unity projects for the Facebook-Games platform
url=MacEditorTargetInstaller/UnitySetup-Facebook-Games-Support-for-Editor-2018.4.0f1.pkg
install=false
mandatory=false
size=46835742
installedsize=111566000
requires_unity=true
md5=0aa3e9b0ec4942e783f63d768b8252f0
optsync_windows=Windows
optsync_webgl=WebGL"#;
        let test_ini_1 = StringReader::new(test_ini);
        let version = Manifest::read_manifest_version(test_ini_1).expect("a version from manifest");
        let test_ini_2 = StringReader::new(test_ini);
        let manifest = Manifest::from_reader(&version, test_ini_2).expect("a manifest from reader");
        assert!(manifest.get(Component::Android).is_some());
        assert!(manifest.get(Component::Ios).is_some());
        assert!(manifest.get(Component::Editor).is_some());
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
