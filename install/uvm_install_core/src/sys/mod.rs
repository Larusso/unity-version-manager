pub mod shared;
cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        mod mac;
        pub use self::mac::*;
    } else if #[cfg(target_os = "windows")] {
        mod win;
        pub use self::win::*;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        pub use self::linux::*;
    } else {
        compile_error!("uvm doens't compile for this platform");
    }
}
