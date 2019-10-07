use crate::error::*;
use crate::unity::{Version,Component};
use crate::unity::urls::{DownloadURL, IniUrl};
use crate::unity::hub::paths;
use reqwest::Url;
use std::collections::HashMap;
use std::fmt;
use std::fs::{DirBuilder, File};
use std::io::{self, Read, Write};
use super::MD5;
use std::path::Path;
use derive_deref::{Deref, DerefMut};
use std::time::Duration;

//use std::ops::{Deref, DerefMut};

use super::client::CLIENT;

#[derive(Deserialize, Debug, Deref, DerefMut)]
#[serde(transparent)]
pub struct IniManifest(HashMap<Component, IniData>);

#[derive(Deserialize, Debug)]
pub struct IniData {
    pub title: String,
    pub description: String,
    pub url: String,
    #[serde(skip)]
    pub download_url: Option<Url>,
    pub size: u64,
    pub installedsize: u64,
    pub md5: Option<MD5>,
    pub cmd: Option<String>,
    #[serde(with = "de::bool")]
    #[serde(default = "de::bool::default")]
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "eulaurl1")]
    pub eula_url_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "eulalabel1")]
    pub eula_label_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "eulamessage")]
    pub eula_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<Component>,
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl IniManifest {

    pub fn load<V:AsRef<Version>>(version:V) -> Result<IniManifest> {
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

    pub fn from_reader<R: Read, V:AsRef<Version>>(version:V, mut manifest: R) -> Result<IniManifest> {
        let base_url = DownloadURL::new(&version)?;
        let mut ini_data = String::new();
        manifest.read_to_string(&mut ini_data)?;
        let ini_data = Self::cleanup_ini_data(&ini_data);

        Self::read_components(ini_data.as_bytes(), &base_url, version)
    }

    pub fn new<P:AsRef<Path>, V:AsRef<Version>>(version: V, manifest_path: P) -> Result<IniManifest> {
        let manifest = File::open(manifest_path)?;
        Self::from_reader(version, manifest)
    }

    fn get_manifest<V:AsRef<Version>>(version: V) -> Result<IniManifest> {
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

        let manifest_path = cache_dir.join(&format!("{}_manifest.ini", version.as_ref()));

        if !manifest_path.exists() {
            Self::download_manifest(version, manifest_path.to_path_buf())
        } else {
            Self::new(version, manifest_path)
        }
    }

    fn read_components<R: Read, V:AsRef<Version>>(input: R, base_url: &DownloadURL, version:V) -> Result<IniManifest> {
        let version = version.as_ref();
        serde_ini::from_read(input)
            .chain_err(|| UvmErrorKind::ManifestReadError)
            .map(|mut components: IniManifest| {
                for (component, data) in components.iter_mut() {
                    data.download_url = component.download_url(base_url, &data.url, version)
                }
                components
            })
    }

    fn download_manifest<V, P>(version: V, path: P) -> Result<IniManifest>
    where
        V:AsRef<Version>,
        P: AsRef<Path>,
    {
        let version = version.as_ref();
        let ini_url = IniUrl::new(version)?;
        let url = ini_url.into_url();
        debug!("IniManifest URL {}", &url);

        let client = &CLIENT;
        let request = client.get(url).build()?;

        debug!("IniManifest Request:");
        debug!("{:?}", request);
        let mut response = client.execute(request)?;

        debug!("IniManifest Repsonse:");
        debug!("{:?}", response);

        if !response.status().is_success() {
            trace!("{}", response.text()?);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unable to load ini manifest. Status: {}", response.status()),
            )
            .into());
        }

        let body = response.text()?;
        let body = Self::cleanup_ini_data(&body);

        let base_url = DownloadURL::new(version)?;
        let manifest = Self::read_components(body.as_bytes(), &base_url, version)?;

        File::create(path.as_ref())
            .and_then(|mut f| write!(f, "{}", body))
            .unwrap_or_else(|err| {
                warn!(
                    "Error while creating the ini manifest cache file for {}",
                    path.as_ref().display()
                );
                warn!("{}", err);
            });

        Ok(manifest)
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

impl fmt::Display for IniData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.title, self.url)
    }
}

mod de {
    pub mod bool {
        use serde::{Deserialize, Deserializer};
        use std::result;

        pub fn default() -> bool {
            false
        }

        pub fn deserialize<'de, D>(deserializer: D) -> result::Result<bool, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            if s.is_empty() {
                Ok(false)
            } else {
                match s.as_str() {
                    "true" => Ok(true),
                    "false" => Ok(true),
                    _ => Ok(false),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;
    use stringreader::StringReader;

    #[test]
    fn fetch_metadata_for_known_unity_version_does_not_fail() {
        let version = Version::f(2019, 1, 6, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }
        IniManifest::load(&version).unwrap();
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
        assert!(IniManifest::load(&version).is_err());
    }

    #[test]
    fn saves_meta_file_to_cache_dir() {
        let version = Version::f(2019, 1, 7, 1);
        let cache_file = paths::cache_dir()
            .map(|f| f.join(&format!("{}_manifest.ini", version.to_string())))
            .unwrap();
        if cache_file.exists() {
            fs::remove_file(&cache_file).unwrap();
        }

        IniManifest::load(&version).unwrap();
        assert!(cache_file.exists());
    }

    #[test]
    fn downloads_manifest_to_local_path() {
        let tempdir = tempfile::tempdir().unwrap();
        let version = Version::f(2018, 2, 0, 2);
        let path = tempdir
            .path()
            .join(&format!("{}_manifest.ini", version.to_string()));

        IniManifest::download_manifest(&version, &path).unwrap();
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
        let version = IniManifest::read_manifest_version(test_ini).expect("a version from manifest");
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
        let version = IniManifest::read_manifest_version(test_ini_1).expect("a version from manifest");
        let test_ini_2 = StringReader::new(test_ini);
        let manifest = IniManifest::from_reader(&version, test_ini_2).expect("a manifest from reader");
        assert!(manifest.get(&Component::Android).is_some());
        assert!(manifest.get(&Component::Ios).is_some());
        assert!(manifest.get(&Component::Editor).is_some());
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
        assert_eq!(IniManifest::cleanup_ini_data(test_ini), expected_ini);
    }
}
