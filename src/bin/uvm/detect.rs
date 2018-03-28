extern crate console;
extern crate uvm;

use std::env;
use std::process;
use console::style;
use uvm::cli::DetectOptions;

const USAGE: &'static str = "
uvm-detect - Find which version of unity was used to generate a project.

Usage:
  uvm-detect [options] [<project-path>]
  uvm-detect (-h | --help)

Options:
  -r, --recursive               Detects a unity version recursivly from current working directory.
                                With this flag set, the tool returns the first version it finds.
  -v, --verbose                 print more output
  -h, --help                    show this help message and exit
";

fn main() {
    let options: DetectOptions = uvm::cli::get_options(USAGE).unwrap();
    let project_version = uvm::dectect_project_version(
        options
            .project_path()
            .unwrap_or(&env::current_dir().unwrap()),
        Some(options.recursive()),
    ).unwrap_or_else(|err| {
        eprintln!("{}", style(err).red());
        process::exit(1);
    });

    println!("{}", style(project_version.to_string()).green().bold());
}
