use std::path::{Path, PathBuf};
use std::io;
use std::fs;
use std::env;
use std::process;
use console::style;
use std::error::Error;

fn find_in_path<F>(dir: &Path, predicate: &F) -> io::Result<PathBuf>
where
    F: Fn(&fs::DirEntry) -> bool,
{
    dir.read_dir()?
        .filter_map(io::Result::ok)
        .find(predicate)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Item not found in directory: {}", dir.display()),
            )
        })
        .map(|entry| entry.path())
}

pub fn sub_command_path(command_name: &str) -> io::Result<PathBuf> {
    let p = env::current_exe()?;
    let base_search_dir = p.parent().unwrap();
    let command_name = format!("uvm-{}", command_name);

    //first check exe directory
    let comparator = |entry: &fs::DirEntry| {
        !entry.file_type().unwrap().is_dir() && entry.file_name() == command_name[..]
    };

    let command_path = find_in_path(base_search_dir, &comparator);
    if command_path.is_ok() {
        return command_path;
    }

    //check PATH
    if let Ok(path) = env::var("PATH") {
        let paths = path.split(":").map(|s| Path::new(s));
        for path in paths {
            let command_path = find_in_path(path, &comparator);
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

pub fn print_error_and_exit<E, T>(err: E) -> T
where
    E: Error,
{
    eprintln!("{}", style(err).red());
    process::exit(1);
}
