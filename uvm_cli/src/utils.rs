use console::style;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs, io, process};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

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
        }).map(|entry| entry.path())
}

#[cfg(unix)]
fn check_file(entry: &fs::DirEntry) -> io::Result<bool> {
    let metadata = entry.metadata()?;
    let file_name = entry.file_name();
    let file_name = file_name.to_string_lossy();
    if !metadata.is_dir() && file_name.starts_with("uvm-") {
        let metadata = entry.path().metadata().unwrap();
        let process = env::current_exe().unwrap();
        let p_metadata = process.metadata().unwrap();
        let p_uid = p_metadata.uid();
        let p_gid = p_metadata.gid();

        let is_user = metadata.uid() == p_uid;
        let is_group = metadata.gid() == p_gid;

        let permissions = metadata.permissions();
        let mode = permissions.mode();
        return Ok((mode & 0o0001) != 0
            || ((mode & 0o0010) != 0 && is_group)
            || ((mode & 0o0100) != 0 && is_user));
    }
    Ok(false)
}

#[cfg(windows)]
fn check_file(entry: &fs::DirEntry) -> io::Result<bool> {
    let metadata = entry.metadata()?;
    let file_name = entry.file_name();
    let file_name = file_name.to_string_lossy();
    trace!("file_name {}", file_name);
    Ok(!metadata.is_dir() && file_name.starts_with("uvm-") && file_name.ends_with(".exe"))
}

pub fn find_commands_in_path(dir: &Path) -> io::Result<Box<Iterator<Item = PathBuf>>> {
    let result = dir
        .read_dir()?
        .filter_map(io::Result::ok)
        .filter(|entry| check_file(entry).unwrap_or(false))
        .map(|entry| entry.path());
    Ok(Box::new(result))
}

pub fn sub_command_path(command_name: &str) -> io::Result<PathBuf> {
    let p = env::current_exe()?;
    let base_search_dir = p.parent().unwrap();

    #[cfg(windows)]
    let command_name = format!("uvm-{}.exe", command_name);
    #[cfg(unix)]
    let command_name = format!("uvm-{}", command_name);
    debug!("fetch path to subcommand: {}", &command_name);

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
        let split_char = if cfg!(windows) { ";" } else { ":" };

        let paths = path.split(split_char).map(|s| Path::new(s));
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

pub struct UvmSubCommands(Box<Iterator<Item = UvmSubCommand>>);

impl UvmSubCommands {
    fn new() -> io::Result<UvmSubCommands> {
        let p = env::current_exe()?;
        let base_search_dir = p.parent().unwrap();
        let mut iter = find_commands_in_path(base_search_dir).ok();

        if let Ok(path) = env::var("PATH") {
            let paths = path.split(":").map(|s| Path::new(s));
            for path in paths {
                if let Ok(sub_commands) = find_commands_in_path(path) {
                    iter = match iter {
                        Some(i) => Some(Box::new(i.chain(sub_commands))),
                        None => Some(sub_commands),
                    };
                }
            }
        }

        if let Some(i) = iter {
            let m = i.map(|path| UvmSubCommand(path));
            Ok(UvmSubCommands(Box::new(m)))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "not Found"))
        }
    }
}

impl Iterator for UvmSubCommands {
    type Item = UvmSubCommand;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct UvmSubCommand(PathBuf);

impl UvmSubCommand {
    pub fn path(&self) -> PathBuf {
        self.0.clone()
    }
    pub fn command_name(&self) -> String {
        String::from(
            self.0
                .file_name()
                .unwrap()
                .to_string_lossy()
                .split("uvm-")
                .last()
                .unwrap(),
        )
    }

    pub fn description(&self) -> String {
        String::from("")
    }
}

pub fn find_sub_commands() -> io::Result<UvmSubCommands> {
    UvmSubCommands::new()
}

pub fn print_error_and_exit<E, T>(err: E) -> T
where
    E: Error,
{
    eprintln!("{}", style(err).red());
    process::exit(1);
}
