use self::error;
use console::style;
use log::*;

use std::path::PathBuf;
use std::process;
use structopt::{
    clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};
use uvm_core::unity::Component;
use uvm_core::unity::Version;

const SETTINGS: &'static [AppSettings] = &[
    AppSettings::ColoredHelp,
    AppSettings::DontCollapseArgsInUsage,
];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    /// Module to install
    ///
    /// A support module to install. You can list all awailable
    /// modules for a given version using `uvm-modules`
    #[structopt(short, long = "module", number_of_values = 1)]
    modules: Option<Vec<Component>>,

    /// Install also synced modules
    ///
    /// Synced modules are optional dependencies of some Unity modules.
    /// e.g. Android SDK for the android module.
    #[structopt(long = "with-sync")]
    sync: bool,

    /// The api version to install in the form of `6000.0.35f1`
    version: Version,

    /// A directory to install the requested version to
    destination: Option<PathBuf>,

    /// print debug output
    #[structopt(short, long)]
    debug: bool,

    /// print more output
    #[structopt(short, long, parse(from_occurrences))]
    verbose: i32,

    /// Color:.
    #[structopt(short, long, possible_values = &ColorOption::variants(), case_insensitive = true, default_value)]
    color: ColorOption,
}

fn main() {
    let opt = Opts::from_args();

    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

    let version = opt.version;
    let modules = opt.modules;
    let install_sync = opt.sync;
    let destination = opt.destination;

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
