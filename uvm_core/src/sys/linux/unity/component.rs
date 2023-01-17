use crate::unity::Component;
use std::path::{Path, PathBuf};

pub fn installpath(component:Component) -> Option<PathBuf> {
    use Component::*;
    let path = match component {
        Mono | VisualStudio | MonoDevelop => None,
        StandardAssets | Example => Some(""),
        Documentation => Some("Editor/Data/Documentation"),
        ExampleProject  => None,
        Android => Some("Editor/Data/PlaybackEngines/AndroidPlayer"),
        AndroidSdkBuildTools => Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK/build-tools"),
        AndroidSdkPlatforms => Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK/platforms"),
        AndroidSdkPlatformTools | AndroidSdkNdkTools => Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK"),
        AndroidNdk => Some("Editor/Data/PlaybackEngines/AndroidPlayer/NDK"),
        AndroidOpenJdk => Some("Editor/Data/PlaybackEngines/AndroidPlayer/OpenJDK"),
        Ios => Some("Editor/Data/PlaybackEngines/iOSSupport"),
        TvOs => Some("Editor/Data/PlaybackEngines/AppleTVSupport"),
        AppleTV => Some("Editor/Data/PlaybackEngines/AppleTVSupport"),
        Linux | LinuxIL2CPP | LinuxServer => Some("Editor/Data/PlaybackEngines/LinuxStandaloneSupport"),
        Mac | MacIL2CPP | MacMono | MacServer => Some("Editor/Data/PlaybackEngines/MacStandaloneSupport"),
        Samsungtv | SamsungTV => Some("Editor/Data/PlaybackEngines/STVPlayer"),
        Tizen => Some("Editor/Data/PlaybackEngines/TizenPlayer"),
        Vuforia | VuforiaAR => Some("Editor/Data/PlaybackEngines/VuforiaSupport"),
        WebGl => Some("Editor/Data/PlaybackEngines/WebGLSupport"),
        Windows | WindowsMono | WindowsServer => Some("Editor/Data/PlaybackEngines/WindowsStandaloneSupport"),
        Facebook | FacebookGames => Some("Editor/Data/PlaybackEngines/Facebook"),
        Language(_) => Some("Editor/Data/Localization"),
        Lumin => None,
        _ => None,
    };

    path.map(|p| Path::new(p).to_path_buf())
}

pub fn install_location(component: Component) -> Option<PathBuf> {
    use Component::*;
    let path = match component {
        AndroidSdkPlatformTools => {
            Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK/platform-tools")
        }
        AndroidSdkNdkTools => {
            Some("Editor/Data/PlaybackEngines/AndroidPlayer/SDK/tools")
        }
        _ => None,
    };
    path.map(|p| Path::new(p).to_path_buf()).or_else(|| installpath(component))
}

pub fn selected(component:Component) -> bool {
    use Component::*;
    match component {
        Documentation => false,
        _ => false
    }
}
