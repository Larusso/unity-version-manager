use anyhow::Result;

use uvm_cli;
use uvm_core;

use console::style;
use std::env;
use std::path::PathBuf;
use structopt::{
    clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};

const SETTINGS: &'static [AppSettings] = &[
    AppSettings::ColoredHelp,
    AppSettings::DontCollapseArgsInUsage,
];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    /// path to project directory.
    project_path: Option<PathBuf>,

    /// Detects a api version recursivly from current working directory.
    /// With this flag set, the tool returns the first version it finds.
    #[structopt(short, long)]
    recursive: bool,

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

fn main() -> Result<()> {
    let opt = Opts::from_args();

    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

    let project_version = uvm_core::dectect_project_version(
        &opt.project_path.unwrap_or(env::current_dir()?),
        Some(opt.recursive),
    )?;

    println!("{}", style(project_version.to_string()).green().bold());
    Ok(())
}
