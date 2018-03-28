mod launch;
mod utils;
mod uvm;
mod detect;
mod list;

pub use self::detect::*;
pub use self::list::*;
pub use self::launch::*;
pub use self::utils::print_error_and_exit;
pub use self::utils::sub_command_path;
pub use self::uvm::*;

use docopt::Docopt;
use std::convert::From;
use std::str::FromStr;
use unity::Version;
use std::path::{PathBuf};
use std::fmt;
use std::fmt::{Debug, Display};
use std::io;
use serde::de::Deserialize;

// Move this and make it smaller

#[derive(Debug, Deserialize)]
struct UseArguments {
    arg_version: String,
    flag_verbose: bool,
}

#[derive(Debug)]
pub struct Options {}

#[derive(Debug)]
pub struct UseOptions {
    pub version: Version,
    pub verbose: bool,
}

impl From<UseArguments> for UseOptions {
    fn from(a: UseArguments) -> Self {
        UseOptions {
            verbose: a.flag_verbose,
            version: Version::from_str(&a.arg_version).expect("Can't read version parameter"),
        }
    }
}

pub fn get_use_options(usage: &str) -> Option<UseOptions> {
    let args: UseArguments = Docopt::new(usage)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    Some(args.into())
}

pub fn get_options<'a,T>(usage: &str) -> io::Result<T> where
    T: Deserialize<'a>
    {
    Docopt::new(usage)
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .map_err(|e| e.exit())
}
