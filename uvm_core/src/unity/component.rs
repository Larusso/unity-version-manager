use self::Component::*;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use std::str::FromStr;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Deserialize)]
pub enum Component {
    #[serde(rename = "Unity")]
    Editor,
    Mono,
    VisualStudio,
    MonoDevelop,
    Documentation,
    StandardAssets,
    Android,
    #[serde(rename = "iOS")]
    Ios,
    TvOs,
    #[serde(rename = "WebGL")]
    WebGl,
    Linux,
    Windows,
    #[serde(rename = "Windows-Mono")]
    WindowsMono,
    #[serde(other)]
    Unknown,
}

impl Component {
    pub fn iterator() -> Iter<'static, Component> {
        static COMPONENTS: [Component; 12] = [
            Mono,
            VisualStudio,
            MonoDevelop,
            Documentation,
            StandardAssets,
            Android,
            Ios,
            TvOs,
            WebGl,
            Linux,
            Windows,
            WindowsMono,
        ];
        COMPONENTS.into_iter()
    }

    #[cfg(target_os = "macos")]
    pub fn installpath(self) -> Option<PathBuf> {
        let path = match self {
            StandardAssets => Some("Standard Assets"),
            Android => Some("PlaybackEngines/AndroidPlayer"),
            Ios => Some("PlaybackEngines/iOSSupport"),
            TvOs => Some("PlaybackEngines/AppleTVSupport"),
            Linux => Some("PlaybackEngines/LinuxStandaloneSupport"),
            Windows => Some("PlaybackEngines/WindowsStandaloneSupport"),
            WindowsMono => Some("PlaybackEngines/WindowsStandaloneSupport"),
            WebGl => Some("PlaybackEngines/WebGLSupport"),
            _ => None,
        };

        path.map(|p| Path::new(p).to_path_buf())
    }

    #[cfg(target_os = "windows")]
    pub fn installpath(self) -> Option<PathBuf> {
        None
    }

    #[cfg(target_os = "macos")]
    pub fn install_location(self) -> Option<PathBuf> {
        self.installpath()
    }

    #[cfg(target_os = "windows")]
    pub fn install_location(self) -> Option<PathBuf> {
        let path = match self {
            StandardAssets => Some(r"Editor\Standard Assets"),
            Android => Some(r"Editor\Data\PlaybackEngines\AndroidPlayer"),
            Ios => Some(r"Editor\Data\PlaybackEngines\iOSSupport"),
            TvOs => Some(r"Editor\Data\PlaybackEngines\AppleTVSupport"),
            Linux => Some(r"Editor\Data\PlaybackEngines\LinuxStandaloneSupport"),
            //Windows => Some(r"Editor\Data\PlaybackEngines\windowsstandalonesupport"),
            //WindowsMono => Some(r"Editor\Data\PlaybackEngines\windowsstandalonesupport"),
            WebGl => Some(r"Editor\Data\PlaybackEngines\WebGLSupport"),
            _ => None,
        };

        path.map(|p| Path::new(p).to_path_buf())
    }

    pub fn is_installed(self, unity_install_location: &Path) -> bool {
        self.install_location()
            .map(|install_path| unity_install_location.join(install_path))
            .map(|install_path| install_path.exists())
            .unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct ParseComponentError {
    message: String,
}

impl ParseComponentError {
    fn new(message: &str) -> ParseComponentError {
        ParseComponentError {
            message: String::from(message),
        }
    }
}

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseComponentError")
    }
}

impl Error for ParseComponentError {
    fn description(&self) -> &str {
        &self.message[..]
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Component::Editor => write!(f, "editor"),
            Component::Mono => write!(f, "mono"),
            Component::VisualStudio => write!(f, "visual studio"),
            Component::MonoDevelop => write!(f, "mono develop"),
            Component::Documentation => write!(f, "documentation"),
            Component::StandardAssets => write!(f, "standard assets"),
            Component::Android => write!(f, "android"),
            Component::Ios => write!(f, "ios"),
            Component::TvOs => write!(f, "tvos"),
            Component::WebGl => write!(f, "webgl"),
            Component::Linux => write!(f, "linux"),
            Component::Windows => write!(f, "windows"),
            Component::WindowsMono => write!(f, "windows-mono"),
            _ => write!(f, "unknown"),
        }
    }
}

impl FromStr for Component {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standardassets" => Ok(Component::StandardAssets),
            "mono" => Ok(Component::Mono),
            "monodevelop" => Ok(Component::MonoDevelop),
            "visualstudio" => Ok(Component::VisualStudio),
            "ios" => Ok(Component::Ios),
            "android" => Ok(Component::Android),
            "webgl" => Ok(Component::WebGl),
            "linux" => Ok(Component::Linux),
            "windows" => Ok(Component::Windows),
            x => Err(ParseComponentError::new(&format!(
                "Unsupported component {}",
                x
            ))),
        }
    }
}
