#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod other;

mod loader;

#[cfg(target_os = "macos")]
use self::macos as sys;
#[cfg(target_os = "windows")]
use self::windows as sys;
#[cfg(target_os = "linux")]
use self::linux as sys;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
use self::other as sys;

pub use self::sys::install_editor;
pub use self::sys::install_module;
pub use self::loader::Loader;
