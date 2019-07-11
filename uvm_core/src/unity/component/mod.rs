use self::Component::*;
use std::fmt;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use std::str::FromStr;

mod error;
mod sys;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Deserialize)]
pub enum Component {
    #[serde(rename = "Unity")]
    Editor,
    Mono,
    VisualStudio,
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
    #[cfg(windows)]
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
    UniversalWindowsPlatform,
    Samsungtv,
    #[serde(rename = "Samsung-TV")]
    SamsungTV,
    Tizen,
    Vuforia,
    #[serde(rename = "Vuforia-Ar")]
    VuforiaAr,
    Windows,
    #[serde(rename = "Windows-Mono")]
    WindowsMono,
    #[serde(rename = "Windows-IL2CPP")]
    #[cfg(windows)]
    WindowsIL2CCP,
    Facebook,
    #[serde(rename = "Facebook-Games")]
    FacebookGames,
    #[serde(rename = "Facebookgameroom")]
    FacebookGameRoom,
    Lumin,
    #[serde(other)]
    Unknown,
}

impl Component {
    pub fn iterator() -> Iter<'static, Component> {
        #[cfg(windows)]
        const SIZE: usize = 38;
        #[cfg(not(windows))]
        const SIZE: usize = 32;

        static COMPONENTS: [Component; SIZE] = [
            Mono,
            VisualStudio,
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
            Ios,
            TvOs,
            AppleTV,
            Linux,
            LinuxMono,
            Mac,
            MacIL2CPP,
            #[cfg(windows)]
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
            VuforiaAr,
            WebGl,
            Windows,
            WindowsMono,
            #[cfg(windows)]
            WindowsIL2CCP,
            Facebook,
            FacebookGames,
            FacebookGameRoom,
            Lumin,
        ];
        COMPONENTS.iter()
    }

    pub fn installpath(self) -> Option<PathBuf> {
        sys::installpath(self)
    }

    pub fn install_location(self) -> Option<PathBuf> {
        sys::install_location(self)
    }

    pub fn is_installed<P: AsRef<Path>>(self, unity_install_location: P) -> bool {
        let unity_install_location = unity_install_location.as_ref();
        self.install_location()
            .map(|install_path| unity_install_location.join(install_path))
            .map(|install_path| install_path.exists())
            .unwrap_or(false)
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Editor => write!(f, "editor"),
            Mono => write!(f, "mono"),
            VisualStudio => write!(f, "visualstudio"),
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
            Ios => write!(f, "ios"),
            TvOs => write!(f, "tvos"),
            AppleTV => write!(f, "appletv"),
            Linux => write!(f, "linux"),
            LinuxMono => write!(f, "linux-mono"),
            Mac => write!(f, "mac"),
            MacIL2CPP => write!(f, "mac-il2cpp"),
            #[cfg(windows)]
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
            VuforiaAr => write!(f, "vuforia-ar"),
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
            "ios" => Ok(Ios),
            "tvos" => Ok(TvOs),
            "appletv" => Ok(AppleTV),
            "linux" => Ok(Linux),
            "linux-mono" => Ok(LinuxMono),
            "mac" => Ok(Mac),
            "mac-il2cpp" => Ok(MacIL2CPP),
            #[cfg(windows)]
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
            "vuforia-ar" => Ok(VuforiaAr),
            "webgl" => Ok(WebGl),
            "windows" => Ok(Windows),
            "windows-mono" => Ok(WindowsMono),
            #[cfg(windows)]
            "windows-il2cpp" => Ok(WindowsIL2CCP),
            "facebook" => Ok(Facebook),
            "facebook-games" => Ok(FacebookGames),
            "facebookgameroom" => Ok(FacebookGameRoom),
            "lumin" => Ok(Lumin),
            x => Err(error::ParseComponentErrorKind::Unsupported(x.to_string()).into()),
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
        let component:Result<Component> = serde_yaml::from_str(ini_id);
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
        ini_name_vuforia_ar_can_be_deserialized, "Vuforia-Ar" => VuforiaAr,
        ini_name_windows_can_be_deserialized, "Windows" => Windows,
        ini_name_windows_mono_can_be_deserialized, "Windows-Mono" => WindowsMono,
        ini_name_facebook_can_be_deserialized, "Facebook" => Facebook,
        ini_name_facebook_games_can_be_deserialized, "Facebook-Games" => FacebookGames,
        ini_name_facebookgamesroom_can_be_deserialized, "Facebookgameroom" => FacebookGameRoom,
        ini_name_lumin_can_be_deserialized, "Lumin" => Lumin
    }

    #[cfg(windows)]
    valid_component_ini_input! {
        ini_name_mac_mono_can_be_deserialized, "Metro" => Metro,
        ini_name_mac_uwp_il2ccp_can_be_deserialized, "UWP-IL2CPP" => UwpIL2CPP,
        ini_name_mac_uwp_net_can_be_deserialized, "UWP-.NET" => UwpNet,
        ini_name_mac_universal_windows_platform_can_be_deserialized, "UniversalWindowsPlatform" => UniversalWindowsPlatform,
        ini_name_windows_il2cpp_can_be_deserialized, "Windows-IL2CPP" => WindowsIL2CCP
    }
}
