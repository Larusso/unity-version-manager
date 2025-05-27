use crate::commands::error::CommandError;
use log::debug;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::prelude::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};

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

pub struct ExternalCommand {
    command: String,
    arguments: Vec<String>,
}

impl crate::commands::Command for ExternalCommand {
    fn execute(&self) -> crate::commands::Result<()> {
        self.exec()
    }
}

impl ExternalCommand {
    pub fn new(mut args: Vec<String>) -> Self {
        let arguments = args.split_off(1);
        let command = args[0].to_owned();
        Self { command, arguments }
    }

    fn exec(&self) -> crate::commands::Result<()> {
        let command = sub_command_path(&self.command).map_err(|err| {
            CommandError::new(
                format!("failed to find subcommand: {}", self.command),
                1007,
                err.into(),
            )
        })?;
        exec_command(command, self.arguments.clone())
            .map_err(|err| {
                CommandError::new(
                    format!(
                        "external command '{}' with arguments {:#?} failed",
                        self.command, &self.arguments
                    ),
                    1008,
                    err.into(),
                )
            })
            .map(|_| ())
    }
}

// #[cfg(unix)]
// fn check_file(entry: &fs::DirEntry) -> io::Result<bool> {
//     let metadata = entry.metadata()?;
//     let file_name = entry.file_name();
//     let file_name = file_name.to_string_lossy();
//     if !metadata.is_dir() && file_name.starts_with("uvm-") {
//         let metadata = entry.path().metadata().unwrap();
//         let process = env::current_exe().unwrap();
//         let p_metadata = process.metadata().unwrap();
//         let p_uid = p_metadata.uid();
//         let p_gid = p_metadata.gid();
//
//         let is_user = metadata.uid() == p_uid;
//         let is_group = metadata.gid() == p_gid;
//
//         let permissions = metadata.permissions();
//         let mode = permissions.mode();
//         return Ok((mode & 0o0001) != 0
//             || ((mode & 0o0010) != 0 && is_group)
//             || ((mode & 0o0100) != 0 && is_user));
//     }
//     Ok(false)
// }
//
// #[cfg(windows)]
// fn check_file(entry: &fs::DirEntry) -> io::Result<bool> {
//     let metadata = entry.metadata()?;
//     let file_name = entry.file_name();
//     let file_name = file_name.to_string_lossy();
//     trace!("file_name {}", file_name);
//     Ok(!metadata.is_dir() && file_name.starts_with("uvm-") && file_name.ends_with(".exe"))
// }

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

#[cfg(unix)]
pub fn exec_command<C, I, S>(command: C, args: I) -> io::Result<i32>
where
    C: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Err(Command::new(command).args(args).exec())
}

#[cfg(windows)]
pub fn exec_command<C, I, S>(command: C, args: I) -> io::Result<i32>
where
    C: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(command)
        .args(args)
        .spawn()?
        .wait()
        .and_then(|s| {
            s.code().ok_or_else(|| {
                io::Error::new(io::ErrorKind::Interrupted, "Process terminated by signal")
            })
        })
}
