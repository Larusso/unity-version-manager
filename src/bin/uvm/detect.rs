extern crate console;
extern crate uvm;

use std::env;
use std::process;
use console::style;

const USAGE: &'static str = "
uvm-detect - Find which version of unity was used to generate a project.

Usage:
  uvm-detect [options] [<project-path>]
  uvm-detect (-h | --help)

Options:
  -v, --verbose                 print more output
  -h, --help                    show this help message and exit
";


fn main() {
    let o = uvm::cli::get_detect_options(USAGE).unwrap();
    let project_path = o.project_path.unwrap_or(env::current_dir().unwrap());

    let project_version = uvm::dectect_project_version(&project_path).unwrap_or_else(|err| {
        eprintln!("{}", style(err).red());
        process::exit(1);
    });

    println!("{}", style(project_version.to_string()).green().bold());
}
