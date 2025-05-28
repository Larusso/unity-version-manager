mod commands;

use std::error::Error;
use crate::commands::detect::DetectCommand;
use crate::commands::external::ExternalCommand;
use crate::commands::install::InstallArgs;
use crate::commands::launch::LaunchArgs;
use crate::commands::list::ListArgs;
use crate::commands::uninstall::UninstallArgs;
use crate::commands::versions::VersionsArgs;
use clap::{ArgAction, Args, ColorChoice, Parser, Subcommand};
use console::Style;
use flexi_logger::{DeferredNow, Level, LevelFilter, LogSpecification, Logger, Record};
use log::{debug, error, log_enabled, Log};
use std::io;
use log::Level::Info;
use crate::commands::Command;

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
    // Launch(LaunchArgs),
    // List(ListArgs),
    // Install(InstallArgs),
    // Uninstall(UninstallArgs),
    // Versions(VersionsArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

impl Commands {
    fn exec(self) -> commands::Result<()> {
        match self {
            Commands::Detect(detect) => detect.execute(),
            Commands::External(args) => ExternalCommand::new(args).execute(),
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
) -> Result<(Box<dyn Log>, flexi_logger::LoggerHandle), flexi_logger::FlexiLoggerError>
{
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
            .then(|| 2 )
            .unwrap_or(i32::from(cli.global.verbose)),
    )?;
    log::set_boxed_logger(logger)?;

    debug!("{:?}", cli);
    let r = cli.command.exec();
    if r.is_err() {
        let err = r.unwrap_err();
        error!("{}", err);
        if log_enabled!(Info) {
            let mut e: &dyn Error = &err;
            while let Some(source) = e.source() {
                error!("> Caused by: {}", source);
                e = source;
            }
        }
        std::process::exit(err.code() as i32);
    };

    Ok(())
}


