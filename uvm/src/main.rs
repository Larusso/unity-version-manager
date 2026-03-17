mod commands;

use crate::commands::detect::DetectCommand;
#[cfg(feature = "dev-commands")]
use crate::commands::download_modules_json::DownloadModulesJsonCommand;
use crate::commands::external::{exec_command, sub_command_path};
use crate::commands::gc::GcCommand;
use crate::commands::install::InstallArgs;
use crate::commands::launch::LaunchCommand;
use crate::commands::list::ListCommand;
use crate::commands::modules::ModulesCommand;
use crate::commands::uninstall::UninstallArgs;
use crate::commands::version::VersionCommand;
use crate::commands::Command;
use clap::{ArgAction, Args, ColorChoice, Parser, Subcommand};
use console::{style, Style};
use flexi_logger::{DeferredNow, Level, LevelFilter, LogSpecification, Logger, Record};
use indicatif_log_bridge::LogWrapper;
use log::{debug, info, Log};
use std::io;
use std::path::PathBuf;
use std::process;
use unity_hub::error;
use unity_hub::unity::hub::paths;
use uvm_gc::{gc_enabled, GarbageCollector};

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

    /// Enable progress bars and spinners (default in interactive mode)
    #[arg(long, conflicts_with = "no_progress", env = "UVM_PROGRESS")]
    pub progress: bool,

    /// Disable progress bars and spinners
    #[arg(long, conflicts_with = "progress", env = "UVM_NO_PROGRESS")]
    pub no_progress: bool,
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
    Launch(LaunchCommand),
    Modules(ModulesCommand),
    Install(InstallArgs),
    Uninstall(UninstallArgs),
    Version(VersionCommand),
    GC(GcCommand),
    #[cfg(feature = "dev-commands")]
    DownloadModulesJson(DownloadModulesJsonCommand),
    #[command(external_subcommand)]
    External(Vec<String>),
}

impl Commands {
    fn exec(self) -> io::Result<i32> {
        match self {
            Commands::Detect(detect) => detect.execute(),
            Commands::List(list) => list.execute(),
            Commands::Launch(launch) => launch.execute(),
            Commands::Modules(modules) => modules.execute(),
            Commands::Install(install) => with_garbage_collection(install),
            Commands::Uninstall(uninstall) => with_garbage_collection(uninstall),
            Commands::Version(version) => with_garbage_collection(version),
            Commands::GC(gc) => gc.execute(),
            #[cfg(feature = "dev-commands")]
            Commands::DownloadModulesJson(cmd) => cmd.execute(),
            Commands::External(args) => {
                let command = ExternalCommand::from_args(args)?;
                with_garbage_collection(command)
            }
        }
    }
}

struct ExternalCommand {
    command: PathBuf,
    args: Vec<String>,
}

impl ExternalCommand {
    fn from_args(mut args: Vec<String>) -> io::Result<Self> {
        let rest = args.split_off(1);
        let command = sub_command_path(&args[0])?;
        Ok(Self {
            command,
            args: rest,
        })
    }
}

impl Command for ExternalCommand {
    fn execute(&self) -> io::Result<i32> {
        exec_command(self.command.clone(), self.args.clone())
    }
}

fn with_garbage_collection(command: impl Command) -> io::Result<i32> {
    let r = command.execute()?;
    if gc_enabled() {
        eprintln!();
        info!("Running garbage collection");
        GarbageCollector::new(paths::cache_dir().unwrap())
            .with_dry_run(false)
            .collect()
            .unwrap_or_else(|e| {
                log::error!("Error running garbage collection: {}", e);
            });
    }
    Ok(r)
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

    // Set level for uvm crates, but keep dependencies at warn to avoid noise
    log_spec_builder.default(LevelFilter::Warn);
    log_spec_builder.module("uvm", level);
    log_spec_builder.module("uvm_install", level);
    log_spec_builder.module("uvm_detect", level);
    log_spec_builder.module("uvm_gc", level);
    log_spec_builder.module("unity_hub", level);
    log_spec_builder.module("unity_version", level);

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

    write!(write, "{}", style.apply_to(record.args()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    debug!("CLI arguments: {:?}", cli);

    set_colors_enabled(&cli.global.color);

    // Set global progress mode based on flags
    commands::progress::set_progress_mode(cli.global.progress, cli.global.no_progress);

    let verbose_level = cli
        .global
        .debug
        .then(|| 2)
        .unwrap_or(i32::from(cli.global.verbose));

    let max_level = match verbose_level {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let (logger, _log_handle) = set_loglevel(verbose_level)?;
    let multi = commands::progress::global_multi_progress();
    LogWrapper::new((*multi).clone(), logger)
        .try_init()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    log::set_max_level(max_level);

    debug!("{:?}", cli);
    let code = cli.command.exec().unwrap_or_else(|err| {
        eprintln!("{:?}", style(err.to_string()).red());
        1
    });
    process::exit(code)
}
