extern crate console;
extern crate uvm_cli;
extern crate uvm_core;

use console::style;
use std::env;
use std::process;
use uvm_cli::DetectOptions;

const USAGE: &'static str = "
uvm-detect - Find which version of unity was used to generate a project.

Usage:
  uvm-detect [options] [<project-path>]
  uvm-detect (-h | --help)

Options:
  -r, --recursive               Detects a unity version recursivly from current working directory.
                                With this flag set, the tool returns the first version it finds.
  -v, --verbose                 print more output
  --color WHEN                  Coloring: auto, always, never [default: auto]
  -h, --help                    show this help message and exit
";

fn main() {
    let options: DetectOptions = uvm_cli::get_options(USAGE).unwrap();
    let project_version = uvm_core::dectect_project_version(
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
