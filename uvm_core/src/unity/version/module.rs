use crate::sys::unity::version::module::get_android_open_jdk_download_info;
use crate::sys::unity::version::module::get_android_sdk_ndk_download_info;
use crate::unity::component::Category;
use crate::unity::MD5;
use crate::unity::{
    Component, IniManifest, IniData, Localization, Version, VersionType,
};

use std::iter::FromIterator;
use reqwest::{Client, IntoUrl};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use relative_path::{RelativePath, RelativePathBuf};

lazy_static! {
    static ref UNITY_BASE_PATTERN: &'static RelativePath = { RelativePath::new("{UNITY_PATH}") };
}

type ManifestIteratorItem<'a> = ((Component, IniData), &'a Version);

impl AsRef<RelativePath> for UNITY_BASE_PATTERN {
    fn as_ref(&self) -> &RelativePath {
        self.deref()
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Module {
    #[serde(with = "id_serialize")]
    pub id: Component,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "id_serialize_optional_sync")]
    pub sync: Option<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "id_serialize_optional")]
    pub parent: Option<Component>,
    pub name: String,
    pub description: String,
    pub download_url: String,
    pub category: Category,
    pub installed_size: u64,
    pub download_size: u64,
    pub visible: bool,
    pub selected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    destination: Option<RelativePathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rename_to: Option<RelativePathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rename_from: Option<RelativePathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<MD5>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eula_url_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eula_label_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eula_message: Option<String>,
}

use std::cmp::Ordering;

impl PartialEq for Module {
    fn eq(&self, other: &Module) -> bool {
        self.id == other.id
            && self.sync == other.sync
            && self.parent == other.parent
            && self.name == other.name
            && self.description == other.description
            && self.download_url == other.download_url
            && self.category == other.category
            && self.cmd == other.cmd
            && self.visible == other.visible
            && self.selected == other.selected
            && self.destination == other.destination
            && self.rename_to == other.rename_to
            && self.rename_from == other.rename_from
            && self.checksum == other.checksum
            && self.eula_url_1 == other.eula_url_1
            && self.eula_label_1 == other.eula_label_1
            && self.eula_message == other.eula_message
    }
}

impl Hash for Module {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.sync.hash(state);
        self.parent.hash(state);
        self.name.hash(state);
        self.description.hash(state);
        self.download_url.hash(state);
        self.category.hash(state);
        self.cmd.hash(state);
        self.visible.hash(state);
        self.selected.hash(state);
        self.destination.hash(state);
        self.rename_to.hash(state);
        self.rename_from.hash(state);
        self.checksum.hash(state);
        self.eula_url_1.hash(state);
        self.eula_label_1.hash(state);
        self.eula_message.hash(state);
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Module) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Module) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl AsRef<Component> for Module {
    fn as_ref(&self) -> &Component {
        &self.id
    }
}

impl Module {
    #[cfg(not(windows))]
    fn destination(component: Component, _: &str) -> Option<RelativePathBuf> {
        component
            .installpath_rel()
            .map(|mut p| {
                if component == Component::Ios {
                    p.pop();
                };
                p
            })
            .map(|p| {
                if p == RelativePath::new("") {
                    UNITY_BASE_PATTERN.to_relative_path_buf()
                } else if p.to_string().starts_with("/") {
                    //very rare case where the path in the component isn't actually relative
                    p
                } else {
                    UNITY_BASE_PATTERN.join(p)
                }
            })
    }

    #[cfg(windows)]
    fn destination(component: Component, installer_url: &str) -> Option<RelativePathBuf> {
        component
            .installpath_with_installer_url(installer_url)
            .map(|p| {
                if p == Path::new("") {
                    UNITY_BASE_PATTERN.to_path_buf()
                } else if p.to_string().starts_with("/") {
                    //very rare case where the path in the component isn't actually relative
                    p
                } else {
                    UNITY_BASE_PATTERN.join(p)
                }
            })
    }

    fn strip_unity_base_url<P: AsRef<RelativePath>, Q: AsRef<Path>>(path: P, base_dir: Q) -> PathBuf {
        let path = path.as_ref();
        base_dir
            .as_ref()
            .join(&path.strip_prefix(&UNITY_BASE_PATTERN).unwrap_or(path).to_path("."))
    }

    pub fn install_rename_from<P: AsRef<Path>>(&self, base_dir: P) -> Option<PathBuf> {
        self.rename_from
            .as_ref()
            .map(|from| Self::strip_unity_base_url(from, base_dir))
    }

    pub fn install_rename_to<P: AsRef<Path>>(&self, base_dir: P) -> Option<PathBuf> {
        self.rename_to
            .as_ref()
            .map(|to| Self::strip_unity_base_url(to, base_dir))
    }

    pub fn install_rename_from_to<P: AsRef<Path>>(
        &self,
        base_dir: P,
    ) -> Option<(PathBuf, PathBuf)> {
        let base_dir = base_dir.as_ref();
        Some((
            self.install_rename_from(base_dir)?,
            self.install_rename_to(base_dir)?,
        ))
    }

    pub fn install_destination<P: AsRef<Path>>(&self, base_dir: P) -> Option<PathBuf> {
        Some(Self::strip_unity_base_url(
            self.destination.as_ref().map(|destination| {
                if self.id == Component::Ios {
                    destination.join("iOSSupport")
                } else {
                    destination.to_relative_path_buf()
                }
            })?,
            base_dir.as_ref(),
        ))
    }
}

pub struct ModuleBuilder {}

impl ModuleBuilder {
    pub fn from<V: AsRef<Version>>(manifest: IniManifest, version: V) -> Modules {
        let version = version.as_ref();
        let has_documentation =
            manifest.get(&Component::Documentation).is_some() && version.major() < 2018;
        let has_android = manifest.get(&Component::Android).is_some();

        let mut modules: Modules = manifest
            .into_iter()
            .map(|item| (item, version))
            .filter(|((component, _), _)| *component != Component::Editor)
            .filter(|((component, _), _)| {
                if version.major() >= 2018 {
                    *component != Component::Documentation
                } else {
                    true
                }
            })
            .map(Module::from)
            .collect();

        modules.append(&mut Self::generate_missing_modules_for_version(
            version,
            has_documentation,
            has_android,
        ));
        modules
    }

    fn generate_missing_modules_for_version<V: AsRef<Version>>(
        version: V,
        has_documentation: bool,
        has_android: bool,
    ) -> Vec<Module> {
        let version = version.as_ref();
        let mut modules: Vec<Module> = Vec::new();

        if !has_documentation && version.major() >= 2018 {
            modules.push(Self::documentation_module_info(&version));
        }

        if has_android && *version >= Version::a(2019, 1, 0, 1) {
            modules.append(&mut get_android_sdk_ndk_download_info(&version).into_iter().map(|module_part| {
                let mut module = Module::default();
                let component = module_part.component;
                module.id = component;
                module.description = format!("{name} {version}", name = &module_part.name, version = &module_part.version);
                module.name = module_part.name;
                module.category = component.category(&version);
                module.download_size = module_part.download_size;
                module.installed_size = module_part.installed_size;
                module.visible = component.visible();
                module.sync = component.sync();
                module.selected = component.selected();
                module.download_url = module_part.download_url;
                module.destination = Module::destination(component, &module.download_url);
                module.rename_to = module_part.rename_to;
                module.rename_from = module_part.rename_from;
                if module_part.main {
                    module.parent = Some(Component::Android);
                    module.sync = Some(Component::Android);
                    module.eula_url_1 = Some("https://dl.google.com/dl/android/repository/repository2-1.xml".to_string());
                    module.eula_label_1 = Some("Android SDK and NDK License Terms from Google".to_string());
                    module.eula_message = Some("Please review and accept the license terms before downloading and installing Android's SDK and NDK.".to_string());
                }
                module
            }).collect());
        }

        if has_android && *version >= Version::a(2019, 2, 0, 1) {
            let module_part = get_android_open_jdk_download_info(&version);
            let mut module = Module::default();
            let component = module_part.component;
            module.id = component;
            module.description = format!(
                "Android Open JDK {version}",
                version = &module_part.version
            );
            module.name = module_part.name;
            module.category = component.category(&version);
            module.download_size = module_part.download_size;
            module.installed_size = module_part.installed_size;
            module.visible = component.visible();
            module.selected = component.selected();
            module.parent = Some(Component::Android);
            module.sync = Some(Component::Android);
            module.download_url = module_part.download_url;
            module.destination = Module::destination(component, &module.download_url);
            modules.push(module);
        }

        if *version >= Version::a(2018, 1, 0, 1) {
            modules.append(
                &mut Localization::locals(&version)
                    .filter_map(|locale| {
                        let mut module = Module::default();
                        let component = Component::Language(locale);
                        module.id = component;
                        module.description = locale.name().to_string();
                        module.name = locale.name().to_string();
                        module.category = component.category(&version);

                        module.visible = component.visible();
                        module.selected = component.selected();
                        module.download_url = format!(
                            "https://new-translate.unity3d.jp/v1/live/54/{major}.{minor}/{lang_code}",
                            major = version.major(),
                            minor = version.minor(),
                            lang_code = locale.locale()
                        );
                        module.destination = Module::destination(component, &module.download_url);

                        if let Some((content_size, _)) = Self::content_size(&module.download_url) {
                            module.download_size = content_size;
                            module.installed_size = content_size;
                        }

                        if module.download_size == 8 && *version.release_type() == VersionType::Alpha {
                            return None;
                        }

                        Some(module)
                    })
                    .collect(),
            );
        }

        modules
    }

    fn documentation_module_info<V: AsRef<Version>>(version: V) -> Module {
        let version = version.as_ref();
        let mut module = Module::default();
        let component = Component::Documentation;
        module.id = component;
        module.name = "Documentation".to_string();
        module.description = "Offline Documentation".to_string();
        let doc_url_base = "cloudmedia-docs.unity3d.com";

        module.download_url = format!(
            "https://{doc_url}/docscloudstorage/{major}.{minor}/UnityDocumentation.zip",
            doc_url = doc_url_base,
            major = version.major(),
            minor = version.minor()
        );
        module.category = component.category(version);
        module.visible = component.visible();
        module.selected = component.selected();
        module.destination = Module::destination(component, &module.download_url);

        if let Some((content_size, file_size)) = Self::content_size(&module.download_url) {
            module.download_size = content_size;
            module.installed_size = file_size;
        }

        module
    }

    fn content_size<U: IntoUrl>(url: U) -> Option<(u64, u64)> {
        let client = Client::builder()
            .gzip(false)
            .build()
            .expect("a HTTP client");

        client
            .head(url)
            .send()
            .ok()
            .and_then(|response| response.content_length())
            .map(|content_length| {
                (
                    content_length,
                    (content_length as f64 * 2.04).round() as u64,
                )
            })
    }

    pub fn cmd(cmd: Option<String>, category: Category) -> Option<String> {
        cmd.and_then(|cmd| {
            if category != Category::Plugins && category != Category::DevTools {
                return None;
            }

            if cmd.as_str() == "/S /D={INSTDIR}" {
                return None;
            }

            if cmd.is_empty() {
                return None;
            }

            let cmd = cmd.replace(r#""{FILENAME}" "#, "");
            Some(cmd)
        })
    }
}

impl From<ManifestIteratorItem<'_>> for Module {
    fn from(((component, data), version): ManifestIteratorItem) -> Self {
        let mut module = Module::default();
        module.id = component;
        module.name = data.title.clone();
        module.category = component.category(version);
        module.description = data.description.clone();
        module.cmd = ModuleBuilder::cmd(data.cmd, module.category);
        module.download_size = if cfg![windows] {
            data.size * 1024
        } else {
            data.size
        };
        module.installed_size = if cfg![windows] {
            data.installedsize * 1024
        } else {
            data.installedsize
        };
        module.checksum = data.md5;
        module.selected = component.selected();
        module.visible = component.visible();
        module.sync = data.sync.or_else(|| component.sync());
        module.download_url = data.download_url.expect("a download URL").to_string();
        module.destination = Module::destination(component, &module.download_url);
        module.eula_url_1 = data.eula_url_1;
        module.eula_label_1 = data.eula_label_1;
        module.eula_message = data.eula_message;
        module
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Modules(Vec<Module>);

impl Deref for Modules {
    type Target = Vec<Module>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Modules {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Modules {
    type Item = Module;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ModulesMap(HashMap<Component, Module>);

impl Deref for ModulesMap {
    type Target = HashMap<Component, Module>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ModulesMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for ModulesMap {
    type Item = (Component, Module);
    type IntoIter = std::collections::hash_map::IntoIter<Component, Module>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Module> for ModulesMap {
    fn from_iter<I: IntoIterator<Item=Module>>(iter: I) -> Self {
        let mut map:HashMap<_, _> = HashMap::new();
        for m in iter {
            map.insert(m.id, m);
        }

        ModulesMap(map)
    }
}

impl FromIterator<Module> for Modules {
    fn from_iter<I: IntoIterator<Item=Module>>(iter: I) -> Self {
        let mut v:Vec<_> = Vec::new();
        for m in iter {
            v.push(m);
        }

        Modules(v)
    }
}

impl From<Vec<Module>> for ModulesMap
{
    fn from(modules:Vec<Module>) -> Self {
        modules
            .into_iter()
            .collect()
    }
}

impl From<ModulesMap> for Modules {
    fn from(modules: ModulesMap) -> Self {
        modules
            .into_iter()
            .map(|(_, module)| module)
            .collect()
    }
}

impl From<Modules> for ModulesMap {
    fn from(modules: Modules) -> Self {
        modules.into_iter().collect()
    }
}

mod id_serialize {
    use crate::unity::Component;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(c: &Component, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&c.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Component, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Component::from_str(&s).map_err(serde::de::Error::custom)
    }
}

mod id_serialize_optional {
    use crate::unity::Component;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(c: &Option<Component>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match c {
            Some(c) => serializer.serialize_str(&c.to_string()),
            None => serializer.serialize_unit(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Component>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Component::from_str(&s)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

mod id_serialize_optional_sync {
    use crate::unity::Component;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(c: &Option<Component>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match c {
            //Fix special case in module json for sync field
            Some(Component::Android) => serializer.serialize_str("Android Build Support"),
            Some(c) => serializer.serialize_str(&c.to_string()),
            None => serializer.serialize_unit(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Component>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        //Fix special case in module json for sync field
        if s == "Android Build Support" {
            Ok(Some(Component::Android))
        } else {
            Component::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::unity::Manifest;
    use super::*;
    use stringreader::StringReader;

    pub const TEST_INI:&str = r#"[Unity]
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

    #[test]
    fn can_create_modules_from_manifest() {
        let version = Version::f(2018, 4, 0, 1);
        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");
        let _modules: Modules = manifest.into_modules();
    }

    #[test]
    fn can_create_modules_map_from_modules() {
        let version = Version::f(2018, 4, 0, 1);
        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");
        let modules: Modules = manifest.into_modules();
        let _modules_map: ModulesMap = modules.into();
    }

    #[test]
    fn can_create_modules_from_modules_map() {
        let version = Version::f(2018, 4, 0, 1);
        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");

        let modules_1: Modules = manifest.into_modules();

        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");

        let mut modules_2: Modules = manifest.into_modules();

        let modules_map: ModulesMap = modules_1.into();
        let mut modules_3: Modules = modules_map.into();

        assert_eq!(modules_2.sort(), modules_3.sort());
    }
}
