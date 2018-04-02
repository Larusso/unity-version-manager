#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate serde;
#[macro_use]
extern crate uvm_core;
extern crate console;

mod launch;
mod utils;
mod uvm;
mod detect;
mod list;
mod use_version;
mod help;

pub use self::detect::*;
pub use self::launch::*;
pub use self::list::*;
pub use self::help::*;
pub use self::use_version::*;
pub use self::utils::print_error_and_exit;
pub use self::utils::sub_command_path;
pub use self::uvm::*;

use docopt::Docopt;
use serde::de::Deserialize;
use std::ffi::OsStr;
use std::io;
use std::process::Command;

pub trait Options {
    fn verbose(&self) -> bool {
        false
    }
}

pub fn get_options<'a, T>(usage: &str) -> io::Result<T>
where
    T: Deserialize<'a> + Options,
{
    Docopt::new(usage)
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| d.deserialize())
        .map_err(|e| e.exit())
}

pub fn exec_command<C,I,S>(command: C, args: I) -> io::Result<i32>
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
            s.code().ok_or(io::Error::new(
                io::ErrorKind::Interrupted,
                "Process terminated by signal",
            ))
        })
}