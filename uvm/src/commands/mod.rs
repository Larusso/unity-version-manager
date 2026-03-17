use std::io;

pub mod detect;
#[cfg(feature = "dev-commands")]
pub mod download_modules_json;
pub mod external;
pub mod gc;
pub mod install;
pub mod launch;
pub mod list;
pub mod modules;
pub mod presentation;
pub mod progress;
pub mod uninstall;
pub mod version;

pub trait Command {
    fn execute(&self) -> io::Result<i32>;
}
