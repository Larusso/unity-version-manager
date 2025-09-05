mod commands;

use crate::commands::detect::DetectCommand;
use crate::commands::external::{exec_command, sub_command_path};
use crate::commands::list::ListCommand;
use crate::commands::version::VersionCommand;
use clap::{ArgAction, Args, ColorChoice, Parser, Subcommand};
use console::{style, Style};
use flexi_logger::{DeferredNow, Level, LevelFilter, LogSpecification, Logger, Record};
use log::{debug, Log};
use std::io;
use std::process;

#[derive(Debug, Args)]
pub struct GlobalOptions {
    /// print debug output
    #[arg(short, long)]
    debug: bool,

    /// print more output
    #[arg(short, long, action = ArgAction::Count, default_value="0")]
    pub verbose: u8,

    /// Color:.
    #[arg(short, long, value_enum, env = "COLOR_OPTION", default_missing_value("always"), num_args(0..=1), default_value_t = ColorChoice::default()
    )]
    pub color: ColorChoice,
}

#[derive(Parser, Debug)]
#[command(name = "uvm", version, author, about)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Detect(DetectCommand),
    List(ListCommand),
    // Install(InstallArgs),
    // Uninstall(UninstallArgs),
    Version(VersionCommand),
    #[command(external_subcommand)]
    External(Vec<String>),
}

impl Commands {
    fn exec(self) -> io::Result<i32> {
        match self {
            Commands::Detect(detect) => detect.execute(),
            Commands::List(list) => list.execute(),
            Commands::Version(version) => version.execute(),
            Commands::External(mut args) => {
                let rest = args.split_off(1);
                let command = sub_command_path(&args[0])?;
                exec_command(command, rest)
            }
        }
    }
}

fn set_colors_enabled(color: &ColorChoice) {
    use ColorChoice::*;
    match color {
        Never => console::set_colors_enabled(false),
        Always => console::set_colors_enabled(true),
        Auto => (),
    };
}

fn set_loglevel(
    verbose: i32,
) -> Result<(Box<dyn Log>, flexi_logger::LoggerHandle), flexi_logger::FlexiLoggerError> {
    let mut log_spec_builder = LogSpecification::builder();
    let level = match verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::max(),
    };

    log_spec_builder.default(level);
    let log_spec = log_spec_builder.build();
    Logger::with(log_spec).format(format_logs).build()
}

fn format_logs(
    write: &mut dyn io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let style = match record.level() {
        Level::Trace => Style::new().white().dim().italic(),
        Level::Debug => Style::new().white().dim(),
        Level::Info => Style::new().green(),
        Level::Warn => Style::new().yellow(),
        Level::Error => Style::new().red(),
    };

    write
        .write(&format!("{}", style.apply_to(record.args())).into_bytes())
        .map(|_| ())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    debug!("CLI arguments: {:?}", cli);

    set_colors_enabled(&cli.global.color);
    let (logger, _log_handle) = set_loglevel(
        cli.global
            .debug
            .then(|| 2)
            .unwrap_or(i32::from(cli.global.verbose)),
    )?;
    log::set_boxed_logger(logger)?;

    debug!("{:?}", cli);
    let code = cli.command.exec().unwrap_or_else(|err| {
        eprintln!("{:?}", style(err.to_string()).red());
        1
    });
    process::exit(code)
}
