use crate::unity::Component;
use std::path::{Path, PathBuf};

pub fn installpath(_component:Component) -> Option<PathBuf> {
    None
}

pub fn install_location(component:Component) -> Option<PathBuf> {
    use Component::*;
    let path = match component {
        Mono | VisualStudio | ExampleProject | Example | FacebookGameRoom => None,
        MonoDevelop => Some(r""),
        Documentation => Some(r"Editor\Data"),
        StandardAssets => Some(r"Editor"),
        Android => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer"),
        AndroidSdkBuildTools => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK\build-tools"),
        AndroidSdkPlatforms => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK\platforms"),
        AndroidSdkPlatformTools | AndroidSdkNdkTools => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\SDK\platforms"),
        AndroidNdk => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer\NDK"),
        Ios => Some(r"Editor\Data\PlaybackEngines\iOSSupport"),
        TvOs | AppleTV => Some(r"Editor\Data\PlaybackEngines\AppleTVSupport"),
        Linux | LinuxMono => Some(r"Editor\Data\PlaybackEngines\LinuxStandaloneSupport"),
        Mac | MacIL2CPP | MacMono => Some(r"Editor\Data\PlaybackEngines\MacStandaloneSupport"),
        Metro | UwpIL2CPP | UwpNet | UniversalWindowsPlatform => Some(r"Editor\Data\PlaybackEngines\MetroSupport"),
        Samsungtv | SamsungTV => Some(r"Editor\Data\PlaybackEngines\STVPlayer"),
        Tizen => Some(r"Editor\Data\PlaybackEngines\TizenPlayer"),
        Vuforia | VuforiaAr => Some(r"Editor\Data\PlaybackEngines\VuforiaSupport"),
        WebGl => Some(r"Editor\Data\PlaybackEngines\WebGLSupport"),
        Windows | WindowsMono | WindowsIL2CCP => Some(r"Editor\Data\PlaybackEngines\WindowsStandaloneSupport"),
        Facebook | FacebookGames => Some("Editor/Data/PlaybackEngines/Facebook"),
        Lumin => None,
        _ => None,
    };

    path.map(|p| Path::new(p).to_path_buf())
}
