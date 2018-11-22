extern crate console;
extern crate flexi_logger;
extern crate uvm_cli;
extern crate uvm_install_editor;

const USAGE: &str = "
uvm-install-editor - Install a unity editor from given installer.

Usage:
  uvm-install-editor [options] <installer> <destination>
  uvm-install-editor (-h | --help)

Options:
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let options: uvm_install_editor::Options = uvm_cli::get_options(USAGE)?;
    uvm_install_editor::UvmCommand::new()
        .exec(&options)
        .unwrap_or_else(uvm_cli::print_error_and_exit);
    Ok(())
}
