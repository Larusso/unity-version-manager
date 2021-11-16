use crate::unity::Component;
use relative_path::{RelativePath, RelativePathBuf};

pub fn installpath(component: Component) -> Option<RelativePathBuf> {
    use Component::*;
    let path = match component {
        Mono | VisualStudio | FacebookGameRoom => None,
        MonoDevelop | Documentation => Some(""),
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
        Linux | LinuxMono | LinuxIL2CPP | LinuxServer => Some("PlaybackEngines/LinuxStandaloneSupport"),
        Mac | MacIL2CPP | MacServer => Some("Unity.app/Contents/PlaybackEngines/MacStandaloneSupport"),
        Samsungtv | SamsungTV => Some("PlaybackEngines/STVPlayer"),
        Tizen => Some("PlaybackEngines/TizenPlayer"),
        Vuforia | VuforiaAR => Some("PlaybackEngines/VuforiaSupport"),
        WebGl => Some("PlaybackEngines/WebGLSupport"),
        Windows | WindowsMono | WindowsServer => Some("PlaybackEngines/WindowsStandaloneSupport"),
        Facebook | FacebookGames => Some("PlaybackEngines/Facebook"),
        Lumin => Some("PlaybackEngines/LuminSupport"),
        Language(_) => Some("Unity.app/Contents/Localization"),
        _ => None,
    };

    path.map(|p| RelativePath::new(p).to_relative_path_buf())
}

pub fn install_location(component: Component) -> Option<RelativePathBuf> {
    use Component::*;
    let path = match component {
        AndroidSdkPlatformTools => {
            Some("PlaybackEngines/AndroidPlayer/SDK/platform-tools")
        }
        AndroidSdkNdkTools => {
            Some("PlaybackEngines/AndroidPlayer/SDK/tools")
        }
        _ => None,
    };
    path.map(|p| RelativePath::new(p).to_relative_path_buf()).or_else(|| installpath(component))
}

pub fn selected(component: Component) -> bool {
    use Component::*;
    match component {
        MonoDevelop | Documentation | ExampleProject | Example | VisualStudio => false,
        _ => false,
    }
}
