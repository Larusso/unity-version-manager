use crate::sys::unity::version::module::get_android_sdk_ndk_download_info;
use crate::sys::unity::version::module::get_android_open_jdk_download_info;
use crate::unity::MD5;
use crate::unity::{VersionType, Component, Manifest, ManifestIteratorItem, Localization};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash)]
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
    pub category: String,
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

pub type Modules = Vec<Module>;

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
        .map(Module::from).collect();
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
