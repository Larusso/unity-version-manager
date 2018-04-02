extern crate uvm_cli;
extern crate uvm_core;
extern crate console;

use console::style;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::process::exit;
use uvm_cli::HelpOptions;
use uvm_cli::Options;

const USAGE: &'static str = "
uvm-list - Prints help page for command.

Usage:
  uvm-help <command>
  uvm-list (-h | --help)

Options:
  -h, --help        show this help message and exit
";

fn main() {
    let mut args: HelpOptions = uvm_cli::get_options(USAGE).unwrap();
    let command = uvm_cli::sub_command_path(args.command()).unwrap_or_else(uvm_cli::print_error_and_exit);

    let exit_code = Command::new(command)
        .arg("--help")
        .spawn()
        .unwrap_or_else(uvm_cli::print_error_and_exit)
        .wait()
        .and_then(|s| {
            s.code().ok_or(io::Error::new(
                io::ErrorKind::Interrupted,
                "Process terminated by signal",
            ))
        })
        .unwrap_or_else(uvm_cli::print_error_and_exit);

    process::exit(exit_code)
}
