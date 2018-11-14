extern crate console;
extern crate flexi_logger;
extern crate uvm_cli;
extern crate uvm_install;

use console::style;
use console::Term;
use std::io::Write;
use std::process;

const USAGE: &'static str = "
uvm-install - Install specified unity version.

Usage:
  uvm-install [options] <version>
  uvm-install (-h | --help)

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
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() -> std::io::Result<()> {
    let mut stdout = Term::stderr();
    let options: uvm_install::Options = uvm_cli::get_options(USAGE)?;
    uvm_install::UvmCommand::new()
        .exec(options)
        .unwrap_or_else(|err| {
            let message = format!("Unable to install");
            write!(stdout, "{}\n", style(message).red()).ok();
            write!(stdout, "{}\n", style(err).red()).ok();
            process::exit(1);
        });

    stdout.write_line("Finish")
}
