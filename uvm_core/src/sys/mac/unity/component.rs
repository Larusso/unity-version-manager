use crate::unity::Component;
use std::path::{Path, PathBuf};

pub fn installpath(component:Component) -> Option<PathBuf> {
    use Component::*;
    let path = match component {
        Mono | VisualStudio | FacebookGameRoom => None,
        MonoDevelop | Documentation | LinuxMono => Some(""),
        StandardAssets => Some("Standard Assets"),
        ExampleProject | Example => Some("/Users/Shared/Unity"),
        Android => Some("PlaybackEngines/AndroidPlayer"),
        AndroidSdkBuildTools => Some("PlaybackEngines/AndroidPlayer/SDK/build-tools"),
        AndroidSdkPlatforms => Some("PlaybackEngines/AndroidPlayer/SDK/platforms"),
        AndroidSdkPlatformTools | AndroidSdkNdkTools => Some("PlaybackEngines/AndroidPlayer/SDK"),
        AndroidNdk => Some("PlaybackEngines/AndroidPlayer/NDK"),
        AndroidOpenJdk => Some("PlaybackEngines/AndroidPlayer/OpenJDK"),
        Ios => Some("PlaybackEngines/iOSSupport"),
        TvOs => Some("PlaybackEngines/AppleTVSupport"),
        AppleTV => Some("PlaybackEngines/AppleTVSupport"),
        Linux  => Some("PlaybackEngines/LinuxStandaloneSupport"),
        Mac | MacIL2CPP => Some("Unity.app/Contents/PlaybackEngines/MacStandaloneSupport"),
        Samsungtv | SamsungTV => Some("PlaybackEngines/STVPlayer"),
        Tizen => Some("PlaybackEngines/TizenPlayer"),
        Vuforia | VuforiaAR => Some("PlaybackEngines/VuforiaSupport"),
        WebGl => Some("PlaybackEngines/WebGLSupport"),
        Windows | WindowsMono => Some("PlaybackEngines/WindowsStandaloneSupport"),
        Facebook | FacebookGames => Some("PlaybackEngines/Facebook"),
        Lumin => Some("PlaybackEngines/LuminSupport"),
        Language(_) => Some("Unity.app/Contents/Localization"),
        _ => None,
    };

    path.map(|p| Path::new(p).to_path_buf())
}

pub fn install_location(component:Component) -> Option<PathBuf> {
    self::installpath(component)
}

pub fn selected(component:Component) -> bool {
    use Component::*;
    match component {
        MonoDevelop | Documentation | ExampleProject | Example | VisualStudio => true,
        _ => false
    }
}
