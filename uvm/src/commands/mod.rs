use std::io;

pub mod detect;
pub mod list;
pub mod install;
pub mod uninstall;
pub mod version;
pub mod external;
pub mod presentation;
pub mod launch;
pub mod modules;
pub mod gc;

pub trait Command {
    fn execute(&self) -> io::Result<i32>;
}