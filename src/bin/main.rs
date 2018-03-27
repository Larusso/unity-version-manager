extern crate console;
extern crate docopt;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate uvm;

use std::process::Command;
use docopt::Docopt;
use std::env;
use std::process::exit;
use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use console::style;
use std::process;
use std::error::Error;
use uvm::cli;

const USAGE: &'static str = "
uvm - Tool that just manipulates a link to the current unity version

Usage:
  uvm <command> [<args>...]
  uvm (-h | --help)
  uvm --version

Options:
  --version         print version
  -h, --help        show this help message and exit

Commands:
  current           prints current activated version of unity
  detect            find which version of unity was used to generate a project
  launch            launch the current active version of unity
  list              list unity versions available
  use               use specific version of unity
  help              show command help and exit
";

#[derive(Debug, Deserialize)]
struct Arguments {
    arg_command: String,
    arg_args: Option<Vec<String>>,
}

fn main() {
    let args: Arguments = Docopt::new(USAGE)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let command = cli::sub_command_path(&args.arg_command).unwrap_or_else(cli::print_error_and_exit);

    let exit_code = Command::new(command)
        .args(args.arg_args.unwrap_or(Vec::new()))
        .spawn()
        .unwrap_or_else(cli::print_error_and_exit)
        .wait()
        .and_then(|s| {
            s.code().ok_or(io::Error::new(
                io::ErrorKind::Interrupted,
                "Process terminated by signal",
            ))
        })
        .unwrap_or_else(cli::print_error_and_exit);

    process::exit(exit_code)
}
