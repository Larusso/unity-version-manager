#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod other;

use super::*;

#[cfg(target_os = "macos")]
pub use self::mac::*;
#[cfg(target_os = "windows")]
pub use self::win::*;
#[cfg(target_os = "linux")]
pub use self::linux::*;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use self::other::*;
