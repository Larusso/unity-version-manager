use crate::error::*;
use crate::unity::urls::{DownloadURL};
use crate::unity::{Component, Version};
use reqwest::header;
use reqwest::Url;
use std::collections::hash_map::Iter;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use super::*;
use super::ini::IniManifest;

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

#[derive(Debug)]
pub struct Manifest<'a> {
    version: &'a Version,
    base_url: DownloadURL,
    components: IniManifest,
}

impl<'a> Manifest<'a> {
    pub fn load(version: &'a Version) -> Result<Manifest<'a>> {
        let components = IniManifest::load(version)?;
        let base_url = DownloadURL::new(&version)?;

        Ok(Manifest {
            version,
            base_url,
            components,
        })
    }

    pub fn read_manifest_version_from_path<P: AsRef<Path>>(manifest_path: P) -> Result<Version> {
        IniManifest::read_manifest_version_from_path(manifest_path)
    }

    pub fn read_manifest_version<R: Read>(reader: R) -> Result<Version> {
        IniManifest::read_manifest_version(reader)
    }

    pub fn from_reader<R: Read>(version: &'a Version, manifest: R) -> Result<Manifest<'a>> {
        let base_url = DownloadURL::new(&version)?;
        let components = IniManifest::from_reader(version, manifest)?;

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
}

impl From<Manifest<'_>> for IniManifest {
    fn from(manifest:Manifest) -> Self {
        manifest.components
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    use std::fs;
    use stringreader::StringReader;
    use crate::unity::hub::paths;

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
        let expected_url = "https://download.unity3d.com/download_unity/f2970305fe1c/LinuxEditorInstaller/Unity-2019.1.6f1.tar.xz";
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
    fn cleanup_ini_data_when_read_with_reader() {
        let test_ini = r#"[Unity]
title=Unity 2018.4.0f1
description=Unity Editor
url=MacEditorInstaller/Unity.pkg
install=true
mandatory=false
size=989685761
installedsize=2576390000
ver sio n=2018.4.0f1
md5=822c52aa75af582318c5d0ef94137f40"#;
        let test_ini_1 = StringReader::new(test_ini);
        let version = Manifest::read_manifest_version(test_ini_1).expect("a version from manifest");
        let test_ini_2 = StringReader::new(test_ini);
        let manifest = Manifest::from_reader(&version, test_ini_2).expect("a manifest from reader");
        assert!(manifest.get(Component::Editor).is_some());
        let c = manifest.get(Component::Editor).unwrap();
        assert_eq!(c.description, "Unity Editor".to_string());
    }
}
