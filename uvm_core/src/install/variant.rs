use std::fmt;
use unity::Component;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum InstallVariant {
    Android,
    Ios,
    WebGl,
    Linux,
    Windows,
    WindowsMono,
    Editor,
}

impl fmt::Display for InstallVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InstallVariant::Android => write!(f, "android"),
            InstallVariant::Ios => write!(f, "ios"),
            InstallVariant::WebGl => write!(f, "webgl"),
            InstallVariant::Linux => write!(f, "linux"),
            InstallVariant::Windows => write!(f, "windows"),
            InstallVariant::WindowsMono => write!(f, "windows-mono"),
            _ => write!(f, "editor"),
        }
    }
}

impl From<Component> for InstallVariant {
    fn from(component: Component) -> Self {
        match component {
            Component::Android => InstallVariant::Android,
            Component::Ios => InstallVariant::Ios,
            Component::WebGl => InstallVariant::WebGl,
            Component::Linux => InstallVariant::Linux,
            Component::Windows => InstallVariant::Windows,
            Component::WindowsMono => InstallVariant::WindowsMono,
            _ => InstallVariant::Editor,
        }
    }
}

impl From<InstallVariant> for Component {
    fn from(component: InstallVariant) -> Self {
        match component {
            InstallVariant::Android => Component::Android,
            InstallVariant::Ios => Component::Ios,
            InstallVariant::WebGl => Component::WebGl,
            InstallVariant::Linux => Component::Linux,
            InstallVariant::Windows => Component::Windows,
            InstallVariant::WindowsMono => Component::WindowsMono,
            InstallVariant::Editor => Component::Editor,
        }
    }
}
