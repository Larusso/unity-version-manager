#[macro_use]
extern crate uvm_cli;
extern crate uvm_install2;
extern crate flexi_logger;
extern crate console;

#[macro_use]
extern crate log;

use console::style;
use std::process;
use console::Term;

const USAGE: &'static str = "
uvm-install2 - Install specified unity version.

Usage:
  uvm-install2 [options] <version> [<destination>]
  uvm-install2 (-h | --help)

Options:
  -a, --all         install all support packages
  --android         install android support for editor
  --ios             install ios support for editor
  --webgl           install webgl support for editor
  --mobile          install mobile support (android, ios, webgl)
  --linux           install linux support for editor
  --windows         install windows support for editor
  --desktop         install desktop support (linux, windows)
  -v, --verbose     print more output
  -d, --debug       print debug output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit

Arguments:
  <version>         The unity version to install in the form of `2018.1.0f3`
  <destination>     A directory to install the requested version to
";

fn main() -> std::io::Result<()> {
    let stdout = Term::stderr();
    let options:uvm_install2::Options = uvm_cli::get_options(USAGE)?;
    uvm_install2::UvmCommand::new().exec(options)
        .unwrap_or_else(|err| {
            let message = format!("Failure during installation");
            stdout.write_line(&format!("{}",style(message).red())).ok();
            info!("{}", &format!("{}",style(err).red()));
            process::exit(1);
        });

    stdout.write_line(&format!("{}", style("Finish").green().bold()))
}
