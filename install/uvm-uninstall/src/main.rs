extern crate uvm_cli;
extern crate uvm_uninstall;

use uvm_uninstall::UninstallOptions;

const USAGE: &'static str = "
uvm-install - Uninstall specified unity version.

Usage:
  uvm-uninstall [options] <version>
  uvm-uninstall (-h | --help)

Options:
  -a, --all         uninstall all support packages
  --android         uninstall android support for editor
  --ios             uninstall ios support for editor
  --webgl           uninstall webgl support for editor
  --mobile          uninstall mobile support (android, ios, webgl)
  --linux           uninstall linux support for editor
  --windows         uninstall windows support for editor
  --desktop         uninstall desktop support (linux, windows)
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let options:UninstallOptions = uvm_cli::get_options(USAGE)?;
    uvm_uninstall::UvmCommand::new().exec(options)
}
