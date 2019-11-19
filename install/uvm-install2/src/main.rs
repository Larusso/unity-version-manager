use clap::{arg_enum, crate_authors, crate_version, value_t, values_t, App, Arg};
use console::style;
use console::Style;
use flexi_logger::writers::*;
use flexi_logger::*;
use log::*;
use std::fs::DirBuilder;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use uvm_core::unity::Component;
use uvm_core::unity::Version;

fn main() {
    let matches = App::new("uvm-install2")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Install specified unity version.")
        .arg(
            Arg::with_name("module")
                .help("Module to install")
                .long_help(
                    "A support module to install. You can list all awailable
modules for a given version using `uvm-modules`",
                )
                .long("module")
                .short("m")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("version")
                .help("The unity version to install in the form of `2018.1.0f3`")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("destination")
                .help("A directory to install the requested version to")
                .required(false)
                .index(2),
        )
        .arg(
            Arg::with_name("debug")
                .help("print debug output")
                .long("debug")
                .short("d"),
        )
        .arg(
            Arg::with_name("sync")
                .help("install also synced modules")
                .long("with-sync"),
        )
        .arg(
            Arg::with_name("verbose")
                .help("print more output")
                .long("verbose")
                .short("v"),
        )
        .arg(
            Arg::with_name("log-dir")
                .help("path to log")
                .long("log-dir")
                .env("UVM_LOG_DIR"),
        )
        .arg(
            Arg::with_name("color")
                .help("Coloring")
                .long("color")
                .takes_value(true)
                .possible_values(&["auto", "always", "never"])
                .default_value("auto"),
        )
        .get_matches();

    let version = value_t!(matches, "version", Version).unwrap();
    let destination = value_t!(matches, "destination", PathBuf).ok();
    let modules = values_t!(matches, "module", Component).ok();

    let install_sync = matches.is_present("sync");

    match value_t!(matches, "color", ColorOption).unwrap() {
        ColorOption::Never => console::set_colors_enabled(false),
        ColorOption::Always => console::set_colors_enabled(true),
        ColorOption::Auto => (),
    };

    let strerr_dup = if matches.is_present("debug") {
        Duplicate::Debug
    } else if matches.is_present("verbose") {
        Duplicate::Info
    } else {
        Duplicate::Error
    };

    let log_dir = matches.value_of("log-dir").map(|dir| {
        Path::new(dir).to_path_buf()
    }).or_else(|| {
        default_log_dir()
    });

    if let Some(ref log_dir) = log_dir {
        DirBuilder::new().recursive(true).create(&log_dir).unwrap();
    }

    setup_logger(log_dir, strerr_dup);

    uvm_install2::install(version, modules.as_ref(), install_sync, destination).unwrap_or_else(
        |err| {
            error!("Failure during installation");
            error!("{}", &err);

            for e in err.iter().skip(1) {
                debug!("{}", &format!("caused by: {}", style(&e).red()));
            }

            if let Some(backtrace) = err.backtrace() {
                debug!("backtrace: {:?}", backtrace);
            }
            process::exit(1);
        },
    );

    eprintln!("{}", style("Finish").green().bold())
}

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum ColorOption {
        Auto,
        Always,
        Never,
    }
}

#[cfg(target_os = "linux")]
fn default_log_dir() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
fn default_log_dir() -> Option<PathBuf> {
    dirs_2::home_dir().map(|p| p.join(".uvm/logs").to_path_buf())
}

#[cfg(target_os = "macos")]
fn default_log_dir() -> Option<PathBuf> {
    dirs_2::home_dir().map(|p| p.join("Library/Logs/UnityVersionManager").to_path_buf())
}

#[cfg(target_os = "linux")]
fn setup_logger(log_dir: Option<PathBuf>, stderr_dup: Duplicate) {
    let log_spec = LogSpecification::default(LevelFilter::Warn)
        .module("uvm_core", LevelFilter::Trace)
        .module("uvm_install2", LevelFilter::Trace)
        .module("uvm_install_core", LevelFilter::Trace)
        .build();

    let mut logger = Logger::with(log_spec)
        .format_for_files(flexi_logger::detailed_format)
        .format_for_stderr(format_logs)
        .duplicate_to_stderr(stderr_dup)
        .rotate(
            Criterion::Size(10_000_000),
            Naming::Numbers,
            Cleanup::KeepLogFiles(10),
        );

    let syslog_connector = SyslogConnector::try_datagram("/dev/log")
        .or_else(|_| SyslogConnector::try_datagram("/var/run/syslog"))
        .unwrap();

    let sys_log_writer = SyslogWriter::try_new(
        SyslogFacility::LocalUse1,
        None,
        LevelFilter::Debug,
        "uvm-install2".to_string(),
        syslog_connector,
    )
    .unwrap();

    if let Some(log_dir) = log_dir {
        logger = logger
            .log_target(LogTarget::FileAndWriter(sys_log_writer))
            .directory(log_dir);
    } else {
        logger = logger.log_target(LogTarget::Writer(sys_log_writer));
    }
    logger.start().unwrap();

    warn!("BIG BAD ERROR");
}

#[cfg(not(target_os = "linux"))]
fn setup_logger(log_dir: Option<PathBuf>, stderr_dup: Duplicate) {
    let log_spec = LogSpecification::default(LevelFilter::Warn)
        .module("uvm_core", LevelFilter::Trace)
        .module("uvm_install2", LevelFilter::Trace)
        .module("uvm_install_core", LevelFilter::Trace)
        .build();

    let mut logger = Logger::with(log_spec)
        .format_for_files(flexi_logger::detailed_format)
        .format_for_stderr(format_logs)
        .duplicate_to_stderr(stderr_dup)
        .rotate(
            Criterion::Size(10_000_000),
            Naming::Numbers,
            Cleanup::KeepLogFiles(10),
        );

    if let Some(log_dir) = log_dir {
        logger = logger
            .log_target(LogTarget::File)
            .directory(log_dir);
    } else {
        logger = logger.log_target(LogTarget::DevNull);
    }
    logger.start().unwrap();

    warn!("BIG BAD ERROR");
}

fn format_logs(
    writer: &mut dyn io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), io::Error> {
    let style = match record.level() {
        Level::Trace => Style::new().white().dim().italic(),
        Level::Debug => Style::new().white().dim(),
        Level::Info => Style::new().white(),
        Level::Warn => Style::new().yellow(),
        Level::Error => Style::new().red(),
    };

    writer
        .write(&format!("{}", style.apply_to(record.args())).into_bytes())
        .map(|_| ())
}
