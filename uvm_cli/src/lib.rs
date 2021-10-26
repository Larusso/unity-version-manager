#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod detect;
mod utils;
mod uvm;
use flexi_logger::{DeferredNow, Level, LevelFilter, LogSpecification, Logger, Record};

pub use self::detect::*;
pub use self::utils::find_sub_commands;
pub use self::utils::print_error_and_exit;
pub use self::utils::sub_command_path;
pub use self::uvm::*;
use console::Style;

use std::ffi::OsStr;
use std::io;
use std::process::Command;

pub use cli_core::{get_options, ColorOption, Options};
pub mod options;

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

pub fn set_colors_enabled(color: &options::ColorOption) {
    use options::ColorOption::*;
    match color {
        Never => console::set_colors_enabled(false),
        Always => console::set_colors_enabled(true),
        Auto => (),
    };
}

pub fn set_loglevel(verbose: i32) {
    let log_spec_builder = match verbose {
        0 => LogSpecification::default(LevelFilter::Warn),
        1 => LogSpecification::default(LevelFilter::Info),
        2 => LogSpecification::default(LevelFilter::Debug),
        _ => LogSpecification::default(LevelFilter::max()),
    };

    let log_spec = log_spec_builder.build();
    Logger::with(log_spec).format(format_logs).start().unwrap();
}

pub fn format_logs(
    write: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let style = match record.level() {
        Level::Trace => Style::new().white().dim().italic(),
        Level::Debug => Style::new().white().dim(),
        Level::Info => Style::new().white(),
        Level::Warn => Style::new().yellow(),
        Level::Error => Style::new().red(),
    };

    write
        .write(&format!("{}", style.apply_to(record.args())).into_bytes())
        .map(|_| ())
}
