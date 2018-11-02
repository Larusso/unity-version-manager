extern crate uvm_cli;
extern crate uvm_versions;

use uvm_versions::VersionsOptions;

const USAGE: &'static str = "
uvm-versions - List available Unity versions to install.

Usage:
  uvm-versions [options]
  uvm-versions (-h | --help)

Options:
  -a, --all         list all available versions
  -f, --final       list available final versions
  -b, --beta        list available beta versions
  --alpha           list available alpha versions
  -p, --patch       list available patch versions
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let options:VersionsOptions = uvm_cli::get_options(USAGE)?;
    uvm_versions::UvmCommand::new().exec(options)
}
