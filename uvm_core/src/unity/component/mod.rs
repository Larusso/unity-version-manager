use self::Component::*;
use crate::sys::unity::component as component_impl;
use crate::unity::{Version, Localization};
use crate::unity::urls::DownloadURL;
use std::fmt;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use std::str::FromStr;
use reqwest::Url;
mod error;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Deserialize)]
pub enum Component {
    Language(Localization),
    #[serde(rename = "Unity")]
    Editor,
    Mono,
    VisualStudio,
    #[cfg(windows)]
    VisualStudioProfessionalUnityWorkload,
    #[cfg(windows)]
    VisualStudioEnterpriseUnityWorkload,
    MonoDevelop,
    Documentation,
    StandardAssets,
    ExampleProject,
    Example,
    Android,
    #[serde(rename = "Android-Sdk-Build-Tools")]
    AndroidSdkBuildTools,
    #[serde(rename = "Android-Sdk-Platforms")]
    AndroidSdkPlatforms,
    #[serde(rename = "Android-Sdk-Platform-Tools")]
    AndroidSdkPlatformTools,
    #[serde(rename = "Android-Sdk-Ndk-Tools")]
    AndroidSdkNdkTools,
    #[serde(rename = "Android-Ndk")]
    AndroidNdk,
    #[serde(rename = "Android-Open-Jdk")]
    AndroidOpenJdk,
    #[serde(rename = "iOS")]
    Ios,
    TvOs,
    AppleTV,
    #[serde(rename = "WebGL")]
    WebGl,
    Linux,
    #[serde(rename = "Linux-Mono")]
    LinuxMono,
    Mac,
    #[serde(rename = "Mac-IL2CPP")]
    MacIL2CPP,
    #[serde(rename = "Mac-Mono")]
    MacMono,
    #[cfg(windows)]
    Metro,
    #[serde(rename = "UWP-IL2CPP")]
    #[cfg(windows)]
    UwpIL2CPP,
    #[serde(rename = "UWP-.NET")]
    #[cfg(windows)]
    UwpNet,
    #[cfg(windows)]
    #[serde(rename = "Universal-Windows-Platform")]
    UniversalWindowsPlatform,
    Samsungtv,
    #[serde(rename = "Samsung-TV")]
    SamsungTV,
    Tizen,
    Vuforia,
    #[serde(rename = "Vuforia-AR")]
    VuforiaAR,
    Windows,
    #[serde(rename = "Windows-Mono")]
    WindowsMono,
    #[serde(rename = "Windows-IL2CPP")]
    #[cfg(windows)]
    WindowsIL2CCP,
    Facebook,
    #[serde(rename = "Facebook-Games")]
    FacebookGames,
    #[serde(rename = "FacebookGameroom")]
    FacebookGameRoom,
    Lumin,
    #[serde(other)]
    Unknown,
}

impl Component {
    pub fn iterator() -> Iter<'static, Component> {
        #[cfg(windows)]
        const SIZE: usize = 41;
        #[cfg(not(windows))]
        const SIZE: usize = 34;

        static COMPONENTS: [Component; SIZE] = [
            Mono,
            VisualStudio,
            #[cfg(windows)]
            VisualStudioEnterpriseUnityWorkload,
            #[cfg(windows)]
            VisualStudioProfessionalUnityWorkload,
            MonoDevelop,
            Documentation,
            StandardAssets,
            ExampleProject,
            Example,
            Android,
            AndroidSdkBuildTools,
            AndroidSdkPlatforms,
            AndroidSdkPlatformTools,
            AndroidSdkNdkTools,
            AndroidNdk,
            AndroidOpenJdk,
            Ios,
            TvOs,
            AppleTV,
            Linux,
            LinuxMono,
            Mac,
            MacIL2CPP,
            MacMono,
            #[cfg(windows)]
            Metro,
            #[cfg(windows)]
            UwpIL2CPP,
            #[cfg(windows)]
            UwpNet,
            #[cfg(windows)]
            UniversalWindowsPlatform,
            Samsungtv,
            SamsungTV,
            Tizen,
            Vuforia,
            VuforiaAR,
            WebGl,
            Windows,
            WindowsMono,
            #[cfg(windows)]
            WindowsIL2CCP,
            Facebook,
            FacebookGames,
            FacebookGameRoom,
            Lumin,
            // Language(Localization::Ja),
            // Language(Localization::Ko),
            // Language(Localization::Fr),
            // Language(Localization::Es),
            // Language(Localization::ZhCn),
            // Language(Localization::ZhHant),
            // Language(Localization::ZhHans),
            // Language(Localization::Ru),
        ];
        COMPONENTS.iter()
    }

    #[cfg(windows)]
    pub fn installpath_with_installer_url(self, installer_url:&str) -> Option<PathBuf> {
        use crate::sys::unity::component::InstallerType;
        if installer_url.ends_with(".zip") {
            component_impl::installpath(self, InstallerType::Zip)
        } else {
            component_impl::installpath(self, InstallerType::Exe)
        }
    }

    #[cfg(windows)]
    pub fn installpath(self) -> Option<PathBuf> {
        use crate::sys::unity::component::InstallerType;

        component_impl::installpath(self, InstallerType::Exe)
    }

    #[cfg(not(windows))]
    pub fn installpath(self) -> Option<PathBuf> {
        component_impl::installpath(self)
    }

    pub fn install_location(self) -> Option<PathBuf> {
        component_impl::install_location(self)
    }

    pub fn selected(self) -> bool {
        component_impl::selected(self)
    }

    pub fn visible(self) -> bool {
        match self {
            Mono | FacebookGameRoom | AndroidSdkPlatformTools | AndroidSdkBuildTools | AndroidSdkPlatforms | AndroidNdk => false,
            #[cfg(windows)]
            VisualStudioProfessionalUnityWorkload | VisualStudioEnterpriseUnityWorkload => false,
            _ => true
        }
    }

    pub fn is_installed<P: AsRef<Path>>(self, unity_install_location: P) -> bool {
        let unity_install_location = unity_install_location.as_ref();
        self.install_location()
            .map(|install_path| unity_install_location.join(install_path))
            .map(|install_path| install_path.exists())
            .unwrap_or(false)
    }

    pub fn category<V: AsRef<Version>>(self, version: V) -> String {
        let c = match self {
            MonoDevelop | VisualStudio => "Dev tools",
            Mono | FacebookGameRoom => "Plugins",
            #[cfg(windows)]
            VisualStudioProfessionalUnityWorkload | VisualStudioEnterpriseUnityWorkload => {
                "Plugins"
            }
            Documentation | StandardAssets | ExampleProject | Example => {
                if *version.as_ref() >= Version::a(2018, 2, 0, 0) {
                    "Documentation"
                } else {
                    "Components"
                }
            }
            Language(_) => "Language packs (Preview)",
            _ => "Platforms",
        };

        c.to_string()
    }

    pub fn sync(self) -> Option<String> {
        let s = match self {
            Mono => Some("visualstudio"),
            AndroidSdkNdkTools | AndroidOpenJdk => Some("Android Build Support"),
            AndroidSdkBuildTools | AndroidSdkPlatformTools | AndroidSdkPlatforms | AndroidNdk => Some("android-sdk-ndk-tools"),
            _ => None,
        };
        s.map(|s| s.to_string())
    }

    fn add_version_to_url<V:AsRef<Version>>(self, download_url:&str, version:V) -> String {
        let version = version.as_ref();
        let version_pattern = &format!("-{}", version);
        if !download_url.contains(version_pattern) {
            if download_url.ends_with(".pkg") {
                return download_url.replace(".pkg", &format!("{}.pkg", &version_pattern))
            } else if download_url.ends_with(".exe") {
                return download_url.replace(".exe", &format!("{}.exe", &version_pattern))
            } else if download_url.ends_with(".tar.xz") {
                return download_url.replace(".tar.xz", &format!("{}.tar.xz", &version_pattern))
            }
        }
        download_url.to_string()
    }

    pub fn download_url<V:AsRef<Version>>(self, base_url:&DownloadURL, download_url:&str, version:V) -> Option<Url> {
        if download_url.starts_with("https://") || download_url.starts_with("http://") {
            return Url::parse(download_url).ok()
        }

        match self {
            #[cfg(target_os = "linux")]
            StandardAssets | Example | Documentation => Url::parse(download_url).ok(),
            _ => base_url.join(&self.add_version_to_url(download_url, version)).ok()
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language(l) => write!(f, "language-{}", l.locale()),
            Editor => write!(f, "editor"),
            Mono => write!(f, "mono"),
            VisualStudio => write!(f, "visualstudio"),
            #[cfg(windows)]
            VisualStudioProfessionalUnityWorkload => {
                write!(f, "visualstudioprofessionalunityworkload")
            }
            #[cfg(windows)]
            VisualStudioEnterpriseUnityWorkload => write!(f, "visualstudioenterpriseunityworkload"),
            MonoDevelop => write!(f, "monodevelop"),
            Documentation => write!(f, "documentation"),
            StandardAssets => write!(f, "standardassets"),
            ExampleProject => write!(f, "exampleprojects"),
            Example => write!(f, "example"),
            Android => write!(f, "android"),
            AndroidSdkBuildTools => write!(f, "android-sdk-build-tools"),
            AndroidSdkPlatforms => write!(f, "android-sdk-platforms"),
            AndroidSdkPlatformTools => write!(f, "android-sdk-platform-tools"),
            AndroidSdkNdkTools => write!(f, "android-sdk-ndk-tools"),
            AndroidNdk => write!(f, "android-ndk"),
            AndroidOpenJdk => write!(f, "android-open-jdk"),
            Ios => write!(f, "ios"),
            TvOs => write!(f, "tvos"),
            AppleTV => write!(f, "appletv"),
            Linux => write!(f, "linux"),
            LinuxMono => write!(f, "linux-mono"),
            Mac => write!(f, "mac"),
            MacIL2CPP => write!(f, "mac-il2cpp"),
            MacMono => write!(f, "mac-mono"),
            #[cfg(windows)]
            Metro => write!(f, "metro"),
            #[cfg(windows)]
            UwpIL2CPP => write!(f, "uwp-il2cpp"),
            #[cfg(windows)]
            UwpNet => write!(f, "uwp-.net"),
            #[cfg(windows)]
            UniversalWindowsPlatform => write!(f, "universal-windows-platform"),
            Samsungtv => write!(f, "samsungtv"),
            SamsungTV => write!(f, "samsung-tv"),
            Tizen => write!(f, "tizen"),
            Vuforia => write!(f, "vuforia"),
            VuforiaAR => write!(f, "vuforia-ar"),
            WebGl => write!(f, "webgl"),
            Windows => write!(f, "windows"),
            WindowsMono => write!(f, "windows-mono"),
            #[cfg(windows)]
            WindowsIL2CCP => write!(f, "windows-il2cpp"),
            Facebook => write!(f, "facebook"),
            FacebookGames => write!(f, "facebook-games"),
            FacebookGameRoom => write!(f, "facebookgameroom"),
            Lumin => write!(f, "lumin"),
            Unknown => write!(f, "unknown"),
        }
    }
}

impl FromStr for Component {
    type Err = error::ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mono" => Ok(Mono),
            "visualstudio" => Ok(VisualStudio),
            #[cfg(windows)]
            "visualstudioprofessionalunityworkload" => Ok(VisualStudioProfessionalUnityWorkload),
            #[cfg(windows)]
            "visualstudioenterpriseunityworkload" => Ok(VisualStudioEnterpriseUnityWorkload),
            "monodevelop" => Ok(MonoDevelop),
            "documentation" => Ok(Documentation),
            "standardassets" => Ok(StandardAssets),
            "exampleprojects" => Ok(ExampleProject),
            "example" => Ok(Example),
            "android" => Ok(Android),
            "android-sdk-build-tools" => Ok(AndroidSdkBuildTools),
            "android-sdk-platforms" => Ok(AndroidSdkPlatforms),
            "android-sdk-platform-tools" => Ok(AndroidSdkPlatformTools),
            "android-sdk-ndk-tools" => Ok(AndroidSdkNdkTools),
            "android-ndk" => Ok(AndroidNdk),
            "android-open-jdk" => Ok(AndroidOpenJdk),
            "ios" => Ok(Ios),
            "tvos" => Ok(TvOs),
            "appletv" => Ok(AppleTV),
            "linux" => Ok(Linux),
            "linux-mono" => Ok(LinuxMono),
            "mac" => Ok(Mac),
            "mac-il2cpp" => Ok(MacIL2CPP),
            "mac-mono" => Ok(MacMono),
            #[cfg(windows)]
            "metro" => Ok(Metro),
            #[cfg(windows)]
            "uwp-il2cpp" => Ok(UwpIL2CPP),
            #[cfg(windows)]
            "uwp-.net" => Ok(UwpNet),
            #[cfg(windows)]
            "universal-windows-platform" => Ok(UniversalWindowsPlatform),
            "samsungtv" => Ok(Samsungtv),
            "samsung-tv" => Ok(SamsungTV),
            "tizen" => Ok(Tizen),
            "vuforia" => Ok(Vuforia),
            "vuforia-ar" => Ok(VuforiaAR),
            "webgl" => Ok(WebGl),
            "windows" => Ok(Windows),
            "windows-mono" => Ok(WindowsMono),
            #[cfg(windows)]
            "windows-il2cpp" => Ok(WindowsIL2CCP),
            "facebook" => Ok(Facebook),
            "facebook-games" => Ok(FacebookGames),
            "facebookgameroom" => Ok(FacebookGameRoom),
            "lumin" => Ok(Lumin),
            x => {
                match x.rsplitn(1,'_').next().and_then(|sub| Localization::from_str(sub).ok()) {
                    Some(locale) => Ok(Language(locale)),
                    None => Err(error::ParseComponentErrorKind::Unsupported(x.to_string()).into())
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_string_representation_from_component() {
        for component in Component::iterator() {
            let string_value = component.to_string();
            assert_ne!(string_value, "unknown");

            let component_value = Component::from_str(&string_value).unwrap();
            assert_eq!(&component_value, component);
        }
    }

    #[test]
    fn unknown_components_values_will_be_wrapped() {
        use serde_yaml::Result;
        let ini_id = "Some-Random-Value";
        let component: Result<Component> = serde_yaml::from_str(ini_id);
        assert!(component.is_ok(), "valid input returns a component");
        assert_eq!(component.unwrap(), Component::Unknown);
    }

    macro_rules! valid_component_ini_input {
        ($($id:ident, $input:expr => $component:ident),*) => {
            $(
                #[test]
                fn $id() {
                    use serde_yaml::Result;
                    let ini_id = $input;
                    let component:Result<Component> = serde_yaml::from_str(ini_id);
                    assert_eq!(component.unwrap(), $component);
                }
            )*
        };
    }

    valid_component_ini_input! {
        ini_name_unity_can_be_deserialized, "Unity" => Editor,
        ini_name_mono_can_be_deserialized, "Mono" => Mono,
        ini_name_visualstudio_can_be_deserialized, "VisualStudio" => VisualStudio,
        ini_name_mono_develop_can_be_deserialized, "MonoDevelop" => MonoDevelop,
        ini_name_documentation_can_be_deserialized, "Documentation" => Documentation,
        ini_name_standartassets_can_be_deserialized, "StandardAssets" => StandardAssets,
        ini_name_exampleproject_can_be_deserialized, "ExampleProject" => ExampleProject,
        ini_name_example_can_be_deserialized, "Example" => Example,
        ini_name_android_can_be_deserialized, "Android" => Android,
        ini_name_android_sdk_build_tools_can_be_deserialized, "Android-Sdk-Build-Tools" => AndroidSdkBuildTools,
        ini_name_android_sdk_platforms_can_be_deserialized, "Android-Sdk-Platforms" => AndroidSdkPlatforms,
        ini_name_android_sdk_platform_tools_can_be_deserialized, "Android-Sdk-Platform-Tools" => AndroidSdkPlatformTools,
        ini_name_android_sdk_ndk_tools_can_be_deserialized, "Android-Sdk-Ndk-Tools" => AndroidSdkNdkTools,
        ini_name_android_ndk_can_be_deserialized, "Android-Ndk" => AndroidNdk,
        ini_name_ios_can_be_deserialized, "iOS" => Ios,
        ini_name_tvos_can_be_deserialized, "TvOs" => TvOs,
        ini_name_apple_tv_can_be_deserialized, "AppleTV" => AppleTV,
        ini_name_webgl_can_be_deserialized, "WebGL" => WebGl,
        ini_name_linux_can_be_deserialized, "Linux" => Linux,
        ini_name_linux_mono_can_be_deserialized, "Linux-Mono" => LinuxMono,
        ini_name_mac_can_be_deserialized, "Mac" => Mac,
        ini_name_mac_il2cpp_can_be_deserialized, "Mac-IL2CPP" => MacIL2CPP,
        ini_name_samsungtv_can_be_deserialized, "Samsungtv" => Samsungtv,
        ini_name_samsung_tv_can_be_deserialized, "Samsung-TV" => SamsungTV,
        ini_name_tizen_can_be_deserialized, "Tizen" => Tizen,
        ini_name_vuforia_can_be_deserialized, "Vuforia" => Vuforia,
        ini_name_vuforia_ar_can_be_deserialized, "Vuforia-AR" => VuforiaAR,
        ini_name_windows_can_be_deserialized, "Windows" => Windows,
        ini_name_windows_mono_can_be_deserialized, "Windows-Mono" => WindowsMono,
        ini_name_facebook_can_be_deserialized, "Facebook" => Facebook,
        ini_name_facebook_games_can_be_deserialized, "Facebook-Games" => FacebookGames,
        ini_name_facebookgamesroom_can_be_deserialized, "FacebookGameroom" => FacebookGameRoom,
        ini_name_lumin_can_be_deserialized, "Lumin" => Lumin
    }

    #[cfg(windows)]
    valid_component_ini_input! {
        ini_name_metro_can_be_deserialized, "Metro" => Metro,
        ini_name_visualstudioenterpriseunityworkload_can_be_deserialized, "VisualStudioEnterpriseUnityWorkload" => VisualStudioEnterpriseUnityWorkload,
        ini_name_visualstudioprofessionalunityworkload_can_be_deserialized, "VisualStudioProfessionalUnityWorkload" => VisualStudioProfessionalUnityWorkload,
        ini_name_mac_uwp_il2ccp_can_be_deserialized, "UWP-IL2CPP" => UwpIL2CPP,
        ini_name_mac_uwp_net_can_be_deserialized, "UWP-.NET" => UwpNet,
        ini_name_mac_universal_windows_platform_can_be_deserialized, "Universal-Windows-Platform" => UniversalWindowsPlatform,
        ini_name_windows_il2cpp_can_be_deserialized, "Windows-IL2CPP" => WindowsIL2CCP
    }
}
