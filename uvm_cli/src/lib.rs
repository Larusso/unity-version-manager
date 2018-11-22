#[macro_use]
extern crate serde_derive;
extern crate cli_core;
extern crate console;
extern crate serde;
extern crate uvm_core;
#[macro_use]
extern crate log;
extern crate flexi_logger;

mod detect;
mod help;
mod launch;
mod use_version;
mod utils;
mod uvm;

pub use self::detect::*;
pub use self::help::*;
pub use self::launch::*;
pub use self::use_version::*;
pub use self::utils::find_sub_commands;
pub use self::utils::print_error_and_exit;
pub use self::utils::sub_command_path;
pub use self::uvm::*;

use std::ffi::OsStr;
use std::io;
use std::process::Command;

pub use cli_core::{get_options, ColorOption, Options};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(unix)]
pub fn exec_command<C, I, S>(command: C, args: I) -> io::Result<i32>
where
    C: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Err(Command::new(command).args(args).exec())
}

#[cfg(windows)]
pub fn exec_command<C, I, S>(command: C, args: I) -> io::Result<i32>
where
    C: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(command)
        .args(args)
        .spawn()?
        .wait()
        .and_then(|s| {
            s.code().ok_or_else(|| {
                io::Error::new(io::ErrorKind::Interrupted, "Process terminated by signal")
            })
        })
}
