use crate::unity::Component;
use std::path::{Path, PathBuf};

pub enum InstallerType {
    Exe,
    Zip
}

pub fn installpath(component:Component, installer_type:InstallerType) -> Option<PathBuf> {
    use Component::*;
    use InstallerType::*;
    match installer_type {
        Exe => {
            let path = match component {
                VisualStudio | ExampleProject | Example | FacebookGameRoom | VisualStudioProfessionalUnityWorkload | VisualStudioEnterpriseUnityWorkload => None,
                AndroidSdkPlatformTools | AndroidSdkNdkTools => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK"),
                AndroidSdkBuildTools => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK"),
                AndroidSdkPlatforms => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK"),
                AndroidNdk => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\NDK"),
                AndroidOpenJdk => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\OpenJDK"),
                Language(_) => Some(r"Editor\Data\Localization"),
                _ => Some(r""),
            };

            path.map(|p| Path::new(p).to_path_buf())
        }
        Zip => install_location(component)
    }
}

pub fn install_location(component:Component) -> Option<PathBuf> {
    use Component::*;
    let path = match component {
        VisualStudio | ExampleProject | Example | FacebookGameRoom | VisualStudioProfessionalUnityWorkload | VisualStudioEnterpriseUnityWorkload => None,
        Mono | MonoDevelop => Some(r""),
        Documentation => Some(r"Editor\Data"),
        StandardAssets => Some(r"Editor"),
        Android => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer"),
        AndroidSdkBuildTools => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK\build-tools"),
        AndroidSdkPlatforms => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK\platforms"),
        AndroidSdkPlatformTools | AndroidSdkNdkTools => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK"),
        AndroidNdk => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\NDK"),
        AndroidOpenJdk => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\OpenJDK"),
        Ios => Some(r"Editor\Data\PlaybackEngines\iOSSupport"),
        TvOs | AppleTV => Some(r"Editor\Data\PlaybackEngines\AppleTVSupport"),
        Linux | LinuxMono => Some(r"Editor\Data\PlaybackEngines\LinuxStandaloneSupport"),
        Mac | MacIL2CPP | MacMono => Some(r"Editor\Data\PlaybackEngines\MacStandaloneSupport"),
        Metro | UwpIL2CPP | UwpNet | UniversalWindowsPlatform => Some(r"Editor\Data\PlaybackEngines\MetroSupport"),
        Samsungtv | SamsungTV => Some(r"Editor\Data\PlaybackEngines\STVPlayer"),
        Tizen => Some(r"Editor\Data\PlaybackEngines\TizenPlayer"),
        Vuforia | VuforiaAR => Some(r"Editor\Data\PlaybackEngines\VuforiaSupport"),
        WebGl => Some(r"Editor\Data\PlaybackEngines\WebGLSupport"),
        Windows | WindowsMono | WindowsIL2CCP => Some(r"Editor\Data\PlaybackEngines\WindowsStandaloneSupport"),
        Facebook | FacebookGames => Some(r"Editor\Data\PlaybackEngines\Facebook"),
        Lumin => None,
        Language(_) => Some(r"Editor\Data\Localization"),
        _ => None,
    };

    path.map(|p| Path::new(p).to_path_buf())
}

pub fn selected(component:Component) -> bool {
    use Component::*;
    match component {
        Documentation | StandardAssets | ExampleProject | Example | VisualStudio => true,
        _ => false
    }
}
