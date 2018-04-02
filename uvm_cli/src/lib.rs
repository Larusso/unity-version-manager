#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate docopt;
#[macro_use]
extern crate uvm_core;
extern crate console;

mod launch;
mod utils;
mod uvm;
mod detect;
mod list;
mod use_version;

pub use self::detect::*;
pub use self::launch::*;
pub use self::list::*;
pub use self::use_version::*;
pub use self::utils::print_error_and_exit;
pub use self::utils::sub_command_path;
pub use self::uvm::*;

use docopt::Docopt;
use std::convert::From;
use std::str::FromStr;
use uvm_core::unity::Version;
use std::path::{PathBuf};
use std::fmt;
use std::fmt::{Debug, Display};
use std::io;
use serde::de::Deserialize;

pub fn get_options<'a,T>(usage: &str) -> io::Result<T> where
    T: Deserialize<'a>
    {
    Docopt::new(usage)
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| d.deserialize())
        .map_err(|e| e.exit())
}
