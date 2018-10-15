use std::path::{PathBuf,Path};
use std::str::FromStr;
use std::fmt;
use std::error::Error;
use std::slice::Iter;
use self::Component::*;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Component {
    Editor,
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
}

impl Component {
    pub fn iterator() -> Iter<'static, Component> {
        static COMPONENTS: [Component;  12] = [Mono, VisualStudio, MonoDevelop, Documentation,
        StandardAssets, Android, Ios, TvOs, WebGl, Linux, Windows, WindowsMono];
        COMPONENTS.into_iter()
    }

    pub fn installpath(&self) -> Option<PathBuf> {
        let path = match self {
            StandardAssets => Some("Standard Assets"),
            Android => Some("PlaybackEngines/AndroidPlayer"),
            Ios => Some("PlaybackEngines/iOSSupport"),
            TvOs => Some("PlaybackEngines/AppleTVSupport"),
            Linux => Some("PlaybackEngines/LinuxStandaloneSupport"),
            Windows => Some("PlaybackEngines/WindowsStandaloneSupport"),
            _ => None
        };

        path.map(|p| Path::new(p).to_path_buf())
    }

    pub fn is_installed(&self, unity_install_location:&Path ) -> bool {
        self.installpath()
            .map(|install_path| unity_install_location.join(install_path))
            .map(|install_path| install_path.exists())
            .unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct ParseComponentError {
    message: String
}

impl ParseComponentError {
    fn new(message: &str) -> ParseComponentError {
        ParseComponentError { message: String::from(message) }
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
            x => Err(ParseComponentError::new(&format!("Unsupported component {}", x)))
        }
    }
}
