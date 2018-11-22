extern crate uvm_cli;
extern crate uvm_commands;

use uvm_commands::CommandsOptions;

const USAGE: &str = "
uvm-commands - Lists all available sub commands.

Usage:
  uvm-commands [options]
  uvm-commands (-h | --help)

Options:
  -l, --list        print a list with commands
  -1                single column list
  -p, --path        print only the path to the commands
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let options: CommandsOptions = uvm_cli::get_options(USAGE)?;
    uvm_commands::UvmCommand::new().exec(&options)
}
