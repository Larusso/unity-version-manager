extern crate console;
extern crate uvm_cli;

use std::process;
use uvm_cli::UvmOptions;

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

fn main() {
    let mut args: UvmOptions = uvm_cli::get_options(USAGE).unwrap();
    let command =
        uvm_cli::sub_command_path(args.command()).unwrap_or_else(uvm_cli::print_error_and_exit);
    let exit_code =
        uvm_cli::exec_command(command, args.mut_arguments().take().unwrap_or(Vec::new()))
            .unwrap_or_else(uvm_cli::print_error_and_exit);
    process::exit(exit_code)
}
