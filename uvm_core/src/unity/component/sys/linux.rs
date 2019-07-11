use super::Component;
use std::path::{Path, PathBuf};

pub fn installpath(component:Component) -> Option<PathBuf> {
    use Component::*;
    let path = match component {
        Mono | VisualStudio | MonoDevelop => None,
        Documentation => Some("Editor/Data/Documentation"),
        StandardAssets | ExampleProject | Example => None,
        Android => Some("Editor/Data/PlaybackEngines/AndroidPlayer"),
        AndroidSdkBuildTools => Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK/build-tools"),
        AndroidSdkPlatforms => Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK/platforms"),
        AndroidSdkPlatformTools | AndroidSdkNdkTools => Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK/platforms"),
        AndroidNdk => Some("Editor/Data/PlaybackEngines/AndroidPlayer/NDK"),
        Ios => Some("Editor/Data/PlaybackEngines/iOSSupport"),
        TvOs => Some("Editor/Data/PlaybackEngines/AppleTVSupport"),
        AppleTV => Some("Editor/Data/PlaybackEngines/AppleTVSupport"),
        Linux | LinuxMono => Some("Editor/Data/PlaybackEngines/LinuxStandaloneSupport"),
        Mac | MacIL2CPP => Some("Editor/Data/PlaybackEngines/MacStandaloneSupport"),
        Samsungtv | SamsungTV => Some("Editor/Data/PlaybackEngines/STVPlayer"),
        Tizen => Some("Editor/Data/PlaybackEngines/TizenPlayer"),
        Vuforia | VuforiaAr => Some("Editor/Data/PlaybackEngines/VuforiaSupport"),
        WebGl => Some("Editor/Data/PlaybackEngines/WebGLSupport"),
        Windows | WindowsMono => Some("Editor/Data/PlaybackEngines/WindowsStandaloneSupport"),
        Facebook | FacebookGames => Some("Editor/Data/PlaybackEngines/Facebook"),
        Lumin => None,
        _ => None,
    };

    path.map(|p| Path::new(p).to_path_buf())
}

pub fn install_location(component:Component) -> Option<PathBuf> {
    self::installpath(component)
}
