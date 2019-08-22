use crate::sys::unity::version::module::get_android_open_jdk_download_info;
use crate::sys::unity::version::module::get_android_sdk_ndk_download_info;
use crate::unity::{VersionType, Component, Manifest, ManifestIteratorItem, Localization};
use crate::unity::MD5;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::str::FromStr;
use crate::unity::component::Category;

#[derive(Serialize, Deserialize, Debug, Default, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub name: String,
    pub description: String,
    pub download_url: String,
    pub category: Category,
    pub installed_size: u64,
    pub download_size: u64,
    pub visible: bool,
    pub selected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename_from: Option<String>,
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
        self.id == other.id &&
        self.sync == other.sync &&
        self.parent == other.parent &&
        self.name == other.name &&
        self.description == other.description &&
        self.download_url == other.download_url &&
        self.category == other.category &&
        self.visible == other.visible &&
        self.selected == other.selected &&
        self.destination == other.destination &&
        self.rename_to == other.rename_to &&
        self.rename_from == other.rename_from &&
        self.checksum == other.checksum &&
        self.eula_url_1 == other.eula_url_1 &&
        self.eula_label_1 == other.eula_label_1 &&
        self.eula_message == other.eula_message
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

impl Module {
    #[cfg(not(windows))]
    fn destination(component:Component, _:&str) -> Option<String> {
        component.installpath()
            .map(|mut p| {
                if component == Component::Ios {
                    p.pop();
                };
                p
            })
            .map(|p| {
                if p == Path::new("") {
                    "{UNITY_PATH}".to_string()
                } else {
                    Path::new("{UNITY_PATH}").join(p).display().to_string()
                }
            })
    }

    #[cfg(windows)]
    fn destination(component:Component, installer_url:&str) -> Option<String> {
        component.installpath_with_installer_url(installer_url)
            .map(|p| {
                if p == Path::new("") {
                    "{UNITY_PATH}".to_string()
                } else {
                    let base = Path::new("{UNITY_PATH}");
                    let s = base.join(p).display().to_string();
                    s.as_str().replace(r"\", "/")
                }
            })
    }
}

impl From<ManifestIteratorItem<'_>> for Module {
    fn from(((component, data), version): ManifestIteratorItem) -> Self {
        let id = component.to_string();
        let mut module = Module::default();

        module.id = id;
        module.name = data.title.clone();
        module.category = component.category(version);
        module.description = data.description.clone();
        module.download_size = if cfg![windows] { data.size * 1024 } else { data.size };
        module.installed_size = if cfg![windows] { data.installedsize * 1024 } else { data.installedsize };
        module.checksum = data.md5;
        module.selected = component.selected();
        module.visible = component.visible();
        module.sync = data.sync.map(|c| c.to_string()).or_else(|| component.sync());
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

impl From<Manifest<'_>> for Modules {
    fn from(manifest: Manifest) -> Self {
        let version = manifest.version().to_owned();
        let has_documentation = manifest.get(Component::Documentation).is_some() && version.major() < 2018;
        let has_android = manifest.get(Component::Android).is_some();

        let mut modules: Modules = manifest.into_iter()
        .filter(|((component, _),_)| {
            *component != Component::Editor
        })
        .filter(|((component, _),_)| {
            if version.major() >= 2018 {
                *component != Component::Documentation
            } else {
                true
            }
        })
        .map(Module::from).collect::<Vec<Module>>().into();

        if !has_documentation && version.major() >= 2018 {
            modules.push(documentation_module_info(&version));
        }

        if has_android && version >= Version::a(2019, 1, 0, 1) {
            modules.append(&mut get_android_sdk_ndk_download_info(&version).into_iter().map(|module_part| {
                let mut module = Module::default();
                let component = module_part.component;
                module.id = component.to_string();
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
                    module.parent = Some("android".to_string());
                    module.sync = Some("Android Build Support".to_string());
                    module.eula_url_1 = Some("https://dl.google.com/dl/android/repository/repository2-1.xml".to_string());
                    module.eula_label_1 = Some("Android SDK and NDK License Terms from Google".to_string());
                    module.eula_message = Some("Please review and accept the license terms before downloading and installing Android's SDK and NDK.".to_string());
                }
                module
            }).collect());
        }

        if has_android && version >= Version::a(2019, 2, 0, 1) {
            let module_part = get_android_open_jdk_download_info(&version);
            let mut module = Module::default();
            let component = module_part.component;
            module.id = component.to_string();
            module.description = format!("Android {name} {version}", name = &module_part.name, version = &module_part.version);
            module.name = module_part.name;
            module.category = component.category(&version);
            module.download_size = module_part.download_size;
            module.installed_size = module_part.installed_size;
            module.visible = component.visible();
            module.selected = component.selected();
            module.parent = Some("android".to_string());
            module.sync = Some("Android Build Support".to_string());
            module.download_url = module_part.download_url;
            module.destination = Module::destination(component, &module.download_url);
            modules.push(module);
        }

        if version >= Version::a(2018, 1, 0, 1) {
            modules.append(&mut Localization::locals(&version).filter_map(|locale| {
                let mut module = Module::default();
                let component = Component::Language(locale);
                module.id = component.to_string();
                module.description = locale.name().to_string();
                module.name = locale.name().to_string();
                module.category = component.category(&version);

                module.visible = component.visible();
                module.selected = component.selected();
                module.download_url = format!("https://new-translate.unity3d.jp/v1/live/54/{major}.{minor}/{lang_code}", major = version.major(), minor = version.minor(), lang_code=locale.locale());
                module.destination = Module::destination(component, &module.download_url);

                if let Some((content_size, _)) = content_size(&module.download_url) {
                    module.download_size = content_size;
                    module.installed_size = content_size;
                }

                if module.download_size == 8 && *version.release_type() == VersionType::Alpha {
                    return None
                }

                Some(module)
            }).collect());
        }

        modules
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct ModulesMap(HashMap<Component, Module>);

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

impl From<Modules> for ModulesMap {
    fn from(modules: Modules) -> Self {
        modules.into_iter()
            .filter_map(|module| {
                match Component::from_str(&module.id) {
                    Ok(component) => Some((component, module)),
                    _ => None
                }
            })
            .collect::<HashMap<Component, Module>>().into()
    }
}

impl From<ModulesMap> for Modules {
    fn from(modules: ModulesMap) -> Self {
        modules.into_iter().map(|(_,module)| module).collect::<Vec<Module>>().into()
    }
}

impl From<HashMap<Component,Module>> for ModulesMap {
    fn from(modules: HashMap<Component,Module>) -> Self {
        ModulesMap(modules)
    }
}

impl From<Vec<Module>> for Modules {
    fn from(modules: Vec<Module>) -> Self {
        Modules(modules)
    }
}

use crate::unity::Version;

fn documentation_module_info<V: AsRef<Version>>(version:V) -> Module {
    let version = version.as_ref();
    let mut module = Module::default();
    let component = Component::Documentation;
    module.id = component.to_string();
    module.name = "Documentation".to_string();
    module.description = "Offline Documentation".to_string();
    module.download_url = format!("https://storage.googleapis.com/docscloudstorage/{major}.{minor}/UnityDocumentation.zip", major = version.major(), minor = version.minor());
    module.category = component.category(version);
    module.visible = component.visible();
    module.selected = component.selected();
    module.destination = Module::destination(component, &module.download_url);

    if let Some((content_size, file_size)) = content_size(&module.download_url) {
        module.download_size = content_size;
        module.installed_size = file_size;
    }

    module
}

use reqwest::{Client,IntoUrl};

fn content_size<U: IntoUrl>(url: U) -> Option<(u64,u64)> {
    let client = Client::builder()
        .gzip(false)
        .build()
        .expect("a HTTP client");

    client.head(url).send().ok().and_then(|response| response.content_length())
    .map(|content_length| (content_length, (content_length as f64 * 2.04).round() as u64))
}

#[cfg(test)]
mod tests {
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
        let version = Version::f(2018,4,0,1);
        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");
        let _modules:Modules = manifest.into();
    }

    #[test]
    fn can_create_modules_map_from_modules() {
        let version = Version::f(2018,4,0,1);
        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");
        let modules:Modules = manifest.into();
        let _modules_map:ModulesMap = modules.into();
    }

    #[test]
    fn can_create_modulesfrom_modules_map() {
        let version = Version::f(2018,4,0,1);
        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");

        let modules_1:Modules = manifest.into();

        let test_ini = StringReader::new(TEST_INI);
        let manifest = Manifest::from_reader(&version, test_ini).expect("a manifest from reader");

        let mut modules_2:Modules = manifest.into();

        let modules_map:ModulesMap = modules_1.into();
        let mut modules_3: Modules = modules_map.into();

        assert_eq!(modules_2.sort(), modules_3.sort());
    }
}
