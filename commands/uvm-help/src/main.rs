extern crate console;
extern crate uvm_cli;
extern crate uvm_core;

use std::process;
use uvm_cli::HelpOptions;

const USAGE: &str = "
uvm-help - Prints help page for command.

Usage:
  uvm-help <command>
  uvm-help (-h | --help)

Options:
  -h, --help        show this help message and exit
";

fn main() {
    let args: HelpOptions = uvm_cli::get_options(USAGE).unwrap();
    let command =
        uvm_cli::sub_command_path(args.command()).unwrap_or_else(uvm_cli::print_error_and_exit);

    let exit_code = uvm_cli::exec_command(command, vec!["--help"])
        .unwrap_or_else(uvm_cli::print_error_and_exit);
    process::exit(exit_code)
}
