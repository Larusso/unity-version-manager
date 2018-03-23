extern crate console;
extern crate docopt;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate uvm;

use std::process::Command;
use docopt::Docopt;
use std::env;
use std::process::exit;
use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use console::style;
use std::process;
use std::error::Error;

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

#[derive(Debug, Deserialize)]
struct Arguments {
    arg_command: String,
    arg_args: Option<Vec<String>>,
}

fn adjusted_path() -> String {
    let key = "PATH";
    match env::var(key) {
        Ok(val) => match env::current_exe() {
            Ok(exe_path) => format!("{}:{}", exe_path.parent().unwrap().display(), val),
            Err(_) => val,
        },
        Err(_) => String::from(""),
    }
}

fn search_path<F>(dir: &Path, comp: &F) -> io::Result<PathBuf>
where
    F: Fn(&fs::DirEntry) -> bool,
{
    for entry in dir.read_dir()? {
        let entry = entry?;
        if comp(&entry) {
            return Ok(entry.path());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Item not found in directory: {}", dir.display()),
    ))
}

fn get_command_path(command_name: &str) -> io::Result<PathBuf> {
    let p = env::current_exe()?;
    let base_search_dir = p.parent().unwrap();
    let command_name = format!("uvm-{}", command_name);

    //first check exe directory
    let comparator = |entry: &fs::DirEntry| {
        entry.file_type().unwrap().is_file() && entry.file_name() == command_name[..]
    };

    let command_path = search_path(base_search_dir, &comparator);
    if command_path.is_ok() {
        return command_path;
    }

    //check PATH
    if let Ok(path) = env::var("PATH") {
        let paths = path.split(":").map(|s| Path::new(s));
        for path in paths {
            let command_path = search_path(path, &comparator);
            if command_path.is_ok() {
                return command_path;
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("command not found: {}", command_name),
    ))
}

fn main() {
    let args: Arguments = Docopt::new(USAGE)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    fn print_error_and_exit<E,T> (err: E) -> T
    where E: Error {
        eprintln!("{}", style(err).red());
        process::exit(1);
    };

    let command = get_command_path(&args.arg_command).unwrap_or_else(print_error_and_exit);

    let exit_code = Command::new(command)
        .args(args.arg_args.unwrap_or(Vec::new()))
        .spawn()
        .unwrap_or_else(print_error_and_exit)
        .wait()
        .and_then(|s| {
            s.code().ok_or(io::Error::new(
                io::ErrorKind::Interrupted,
                "Process terminated by signal",
            ))
        })
        .unwrap_or_else(print_error_and_exit);

    process::exit(exit_code)
}
