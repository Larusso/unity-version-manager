use console::style;
use clap::{arg_enum, crate_authors, crate_version, value_t, values_t, App, Arg};
use console::Style;
use flexi_logger::DeferredNow;
use flexi_logger::{Level, LevelFilter, LogSpecification, Logger, Record};
use std::io;
use log::*;
use std::path::PathBuf;
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
        // .unwrap_or_else(|_| vec![Component::Editor]);

    let install_sync = matches.is_present("sync");

    match value_t!(matches, "color", ColorOption).unwrap() {
        ColorOption::Never => console::set_colors_enabled(false),
        ColorOption::Always => console::set_colors_enabled(true),
        ColorOption::Auto => (),
    };

    let mut log_spec_builder = LogSpecification::default(LevelFilter::Warn);
    let log_spec_builder = if matches.is_present("debug") {
        log_spec_builder
            .module("uvm_core", LevelFilter::Trace)
            .module("uvm_install2", LevelFilter::Trace)
            .module("uvm_install_core", LevelFilter::Trace)
    } else if matches.is_present("verbose") {
        log_spec_builder
            .module("uvm_core", LevelFilter::Info)
            .module("uvm_install2", LevelFilter::Info)
            .module("uvm_install_core", LevelFilter::Info)
    } else {
        log_spec_builder
        .module("uvm_core", LevelFilter::Warn)
        .module("uvm_install2", LevelFilter::Warn)
        .module("uvm_install_core", LevelFilter::Warn)
    };

    let log_spec = log_spec_builder.build();
    Logger::with(log_spec).format(format_logs).start().unwrap();

    uvm_install2::install(version, modules.as_ref(), install_sync, destination).unwrap_or_else(|err| {
        error!("Failure during installation");
        error!("{}", &err);

        for e in err.iter().skip(1) {
            debug!("{}", &format!("caused by: {}", style(&e).red()));
        }

        if let Some(backtrace) = err.backtrace() {
            debug!("backtrace: {:?}", backtrace);
        }
        process::exit(1);
    });

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
