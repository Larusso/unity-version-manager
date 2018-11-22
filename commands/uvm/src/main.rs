extern crate console;
extern crate uvm_cli;

use std::process;
use uvm_cli::UvmOptions;

const USAGE: &str = "
uvm - Tool that just manipulates a link to the current unity version

Usage:
  uvm [options] <command> [<args>...]
  uvm (-h | --help)
  uvm --version

Options:
  -v, --verbose     print more output
  -d, --debug       print debug output
  --version         print version
  -h, --help        show this help message and exit

Commands:
  clear             Clear active unity version
  current           Prints current activated version of unity
  detect            Find which version of unity was used to generate a project
  launch            Launch the current active version of unity
  list              List unity versions available
  use               Use specific version of unity
  install           Install specified unity version
  uninstall         Uninstall specified unity version
  versions          List available Unity versions to install
  help              show command help and exit
";

fn main() {
    let mut args: UvmOptions = uvm_cli::get_options(USAGE).unwrap();
    let command =
        uvm_cli::sub_command_path(args.command()).unwrap_or_else(uvm_cli::print_error_and_exit);
    let exit_code = uvm_cli::exec_command(command, args.mut_arguments().take().unwrap_or_default())
        .unwrap_or_else(uvm_cli::print_error_and_exit);
    process::exit(exit_code)
}
