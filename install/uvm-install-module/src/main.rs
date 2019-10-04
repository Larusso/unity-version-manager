use uvm_cli;
use uvm_install_module;

const USAGE: &str = "
uvm-install-module - Install a unity module from given installer.

Usage:
  uvm-install-module [options] <installer> <destination>
  uvm-install-module (-h | --help)

Options:
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let options: uvm_install_module::Options = uvm_cli::get_options(USAGE)?;
    uvm_install_module::UvmCommand::new()
        .exec(&options)
        .unwrap_or_else(uvm_cli::print_error_and_exit);
    Ok(())
}
