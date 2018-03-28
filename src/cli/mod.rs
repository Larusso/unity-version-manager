mod launch;
mod utils;
mod uvm;

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
struct ListArguments {
    flag_verbose: bool,
}

#[derive(Debug, Deserialize)]
struct UseArguments {
    arg_version: String,
    flag_verbose: bool,
}

#[derive(Debug, Deserialize)]
struct DetectArguments {
    arg_project_path: Option<PathBuf>,
    flag_recursive: bool,
    flag_verbose: bool,
}

#[derive(Debug)]
pub struct Options {}

#[derive(Debug)]
pub struct ListOptions {
    pub verbose: bool,
}

#[derive(Debug)]
pub struct UseOptions {
    pub version: Version,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct DetectOptions {
    pub project_path: Option<PathBuf>,
    pub recursive: bool,
    pub verbose: bool,
}

impl From<ListArguments> for ListOptions {
    fn from(a: ListArguments) -> Self {
        ListOptions {
            verbose: a.flag_verbose,
        }
    }
}

impl From<UseArguments> for UseOptions {
    fn from(a: UseArguments) -> Self {
        UseOptions {
            verbose: a.flag_verbose,
            version: Version::from_str(&a.arg_version).expect("Can't read version parameter"),
        }
    }
}

impl From<DetectArguments> for DetectOptions {
    fn from(a: DetectArguments) -> Self {
        DetectOptions {
            recursive: a.flag_recursive,
            verbose: a.flag_verbose,
            project_path: a.arg_project_path,
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

pub fn get_list_options(usage: &str) -> Option<ListOptions> {
    let args: ListArguments = Docopt::new(usage)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    Some(args.into())
}

pub fn get_detect_options(usage: &str) -> Option<DetectOptions> {
    let args: DetectArguments = Docopt::new(usage)
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
