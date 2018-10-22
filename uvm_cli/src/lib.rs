#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate serde;
#[macro_use]
extern crate uvm_core;
extern crate console;
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
pub use self::utils::print_error_and_exit;
pub use self::utils::sub_command_path;
pub use self::utils::find_sub_commands;
pub use self::uvm::*;

use flexi_logger::{Logger, LogSpecification, Record, LevelFilter, Level};
use console::Style;
use docopt::Docopt;
use serde::de::Deserialize;
use std::ffi::OsStr;
use std::io;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::process::CommandExt;


#[derive(PartialEq, Deserialize, Debug)]
pub enum ColorOption {
    Auto,
    Always,
    Never,
}

pub trait Options {
    fn debug(&self) -> bool {
        self.verbose()
    }

    fn verbose(&self) -> bool {
        false
    }

    fn color(&self) -> &ColorOption {
        &ColorOption::Auto
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
        .and_then(|o| {
            set_colors_enabled(&o);
            set_loglevel(&o);
            Ok(o)
        })
}

fn set_colors_enabled<T>(options: &T)
where
    T: Options,
{
    match *options.color() {
        ColorOption::Never => console::set_colors_enabled(false),
        ColorOption::Always => console::set_colors_enabled(true),
        ColorOption::Auto => (),
    };
}

fn set_loglevel<T>(options: &T)
where
    T: Options,
{
    let log_spec_builder = if options.debug() {
        LogSpecification::default(LevelFilter::max())
    }
    else if options.verbose() {
        LogSpecification::default(LevelFilter::Info)
    }
    else {
        LogSpecification::default(LevelFilter::Warn)
    };

    let log_spec = log_spec_builder.build();

    Logger::with(log_spec)
        .format(format_logs)
        .start()
        .unwrap();
}

#[cfg(unix)]
pub fn exec_command<C, I, S>(command: C, args: I) -> io::Result<i32>
where
    C: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Err(Command::new(command)
        .args(args)
        .exec())
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
            s.code().ok_or(io::Error::new(
                io::ErrorKind::Interrupted,
                "Process terminated by signal",
            ))
        })
}

fn format_logs(writer: &mut io::Write, record: &Record) -> Result<(), io::Error> {
    let style = match record.level() {
        Level::Trace => Style::new().white().dim().italic(),
        Level::Debug => Style::new().white().dim(),
        Level::Info => Style::new().white(),
        Level::Warn => Style::new().yellow(),
        Level::Error => Style::new().red()
    };

    writer.write(&format!("{}", style.apply_to(record.args())).into_bytes())
        .map(|_| ())
}
