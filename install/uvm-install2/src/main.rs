use clap::{arg_enum, crate_authors, crate_version, value_t, values_t, App, Arg};
use console::style;
use log::*;
use std::fs::DirBuilder;

use std::path::{Path, PathBuf};
use std::process;
use uvm_core::unity::Component;
use uvm_core::unity::Version;
use uvm_log::{Logger, Duplicate};

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

    let stderr_dup = if matches.is_present("debug") {
        Duplicate::Debug
    } else if matches.is_present("verbose") {
        Duplicate::Info
    } else {
        Duplicate::Error
    };

    let log_dir = matches.value_of("log-dir").map(|dir| {
        Path::new(dir).to_path_buf()
    }).or_else(|| {
        uvm_log::default_log_dir()
    });

    let mut logger = Logger::new().duplicate_to_stderr(stderr_dup);
    if let Some(ref log_dir) = log_dir {
        DirBuilder::new().recursive(true).create(&log_dir).unwrap();
        logger = logger.log_dir(log_dir);
    }

    if let Err(err) = logger.start() {
        eprintln!("unable to start logger \n {}", err);
    }

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
