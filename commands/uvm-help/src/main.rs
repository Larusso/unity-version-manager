#[macro_use]
extern crate serde_derive;
extern crate docopt;

use std::process::Command;
use docopt::Docopt;
use std::env;
use std::process::exit;

const USAGE: &'static str = "
uvm-list - Prints help page for command.

Usage:
  uvm-help <command>
  uvm-list (-h | --help)

Options:
  -h, --help        show this help message and exit
";

#[derive(Debug, Deserialize)]
struct Arguments {
    arg_command: String
}

fn adjusted_path() -> String {
  let key = "PATH";
  match env::var(key) {
    Ok(val) => {
      match env::current_exe() {
        Ok(exe_path) => format!("{}:{}", exe_path.as_path().parent().unwrap().display(), val),
        Err(_) => val,
      }
    },
    Err(_) => String::from(""),
  }
}

fn main() {
  let args: Arguments = Docopt::new(USAGE)
                            .and_then(|d| Ok(d.options_first(true)))
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

  let mut command = Command::new(format!("uvm-{}", args.arg_command));
  command.env("PATH", adjusted_path());
  command.arg("--help");

  let mut process = match command.spawn() {
        Err(_) => panic!("command not found: {}", args.arg_command),
        Ok(process) => process,
  };

  let status = process.wait().unwrap();
  match status.code() {
    Some(code) => exit(code),
    None       => println!("Process terminated by signal")
  }
}
