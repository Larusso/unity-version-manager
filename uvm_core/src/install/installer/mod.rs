#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod other;
#[cfg(target_os = "windows")]
mod windows;

mod loader;

#[cfg(target_os = "macos")]
use self::macos as sys;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use self::other as sys;
#[cfg(target_os = "windows")]
use self::windows as sys;

pub use self::sys::install_editor;
pub use self::sys::install_module;
pub use self::loader::Loader;
