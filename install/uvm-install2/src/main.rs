use std::path::PathBuf;
use clap::{ColorChoice, Parser, ArgAction, arg};
use console::Style;
use flexi_logger::{DeferredNow, LogSpecification, Logger};
use log::{debug, Level, LevelFilter, Log, Record};
use unity_version::Version;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[command(verbatim_doc_comment)]
struct Cli {
    /// Module to install
    ///
    /// A support module to install. You can list all available
    /// modules for a given version using `uvm-modules`
    #[arg(short, long = "module", number_of_values = 1)]
    modules: Option<Vec<String>>,

    /// Install also synced modules
    ///
    /// Synced modules are optional dependencies of some Unity modules.
    /// e.g. Android SDK for the android module.
    #[arg(long = "with-sync")]
    sync: bool,

    /// The api version to install in the form of `2018.1.0f3`
    editor_version: Version,

    /// A directory to install the requested version to
    destination: Option<PathBuf>,

    /// print debug output
    #[arg(short, long)]
    debug: bool,

    /// print more output
    #[arg(short, long, action = ArgAction::Count, default_value="1")]
    pub verbose: u8,

    /// Color:.
    #[arg(short, long, value_enum, env = "COLOR_OPTION", default_missing_value("always"), num_args(0..=1), default_value_t = ColorChoice::default())]
    pub color: ColorChoice,
}


pub fn set_colors_enabled(color: &ColorChoice) {
    use ColorChoice::*;
    match color {
        Never => console::set_colors_enabled(false),
        Always => console::set_colors_enabled(true),
        Auto => (),
    };
}


pub fn set_loglevel(
    verbose: i32,
) -> Result<(Box<dyn Log>, flexi_logger::LoggerHandle), flexi_logger::FlexiLoggerError> {
    let mut log_sepc_builder = LogSpecification::builder();
    let level = match verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::max(),
    };

    log_sepc_builder.default(level);
    let log_spec = log_sepc_builder.build();
    Logger::with(log_spec).format(format_logs).build()
}

pub fn format_logs(
    write: &mut dyn std::io::Write,
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

    set_colors_enabled(&cli.color);
    let (logger, _log_handle) =
        set_loglevel(cli.debug.then(|| 2).unwrap_or(i32::from(cli.verbose)))?;
    log::set_boxed_logger(logger)?;


    let version = cli.editor_version;
    let modules = cli.modules;
    let install_sync = cli.sync;
    let destination = cli.destination;

    eprintln!("Request to install Unity Editor version {} with modules {:?} to destination: {:?}", version, modules, destination);

    uvm_install2::install(version, modules, install_sync, destination)?;

    Ok(())
}