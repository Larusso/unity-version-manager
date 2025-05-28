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

pub fn sub_command_path(command_name: &str) -> io::Result<PathBuf> {
    let p = env::current_exe()?;
    let base_search_dir = p.parent().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "failed to find parent directory of current executable",
    ))?;

    #[cfg(windows)]
    let command_name = format!("uvm-{}.exe", command_name);
    #[cfg(unix)]
    let command_name = format!("uvm-{}", command_name);
    debug!("fetch path to subcommand: {}", &command_name);

    //first check exe directory
    let comparator = |entry: &fs::DirEntry| {
        !entry.path().is_dir() && entry.file_name() == OsStr::new(&command_name)
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
fn exec_command<C, I, S>(command: C, args: I) -> io::Result<i32>
where
    C: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Err(Command::new(command).args(args).exec())
}

#[cfg(windows)]
fn exec_command<C, I, S>(command: C, args: I) -> io::Result<i32>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Command;
    use tempfile::TempDir;

    #[test]
    fn test_external_command_creation() {
        let args = vec!["test".to_string(), "arg1".to_string(), "arg2".to_string()];
        let command = ExternalCommand::new(args);
        assert_eq!(command.command, "test");
        assert_eq!(command.arguments, vec!["arg1", "arg2"]);
    }

    #[test]
    fn test_sub_command_path_not_found() {
        let result = sub_command_path("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_in_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test-file");
        fs::write(&file_path, "test content").unwrap();

        let result = find_in_path(temp_dir.path(), &|entry| {
            entry.file_name() == std::ffi::OsStr::new("test-file")
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), file_path);
    }

    #[test]
    fn test_external_command_execution_invalid() {
        let command = ExternalCommand::new(vec!["nonexistent".to_string()]);
        let result = command.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_with_mock() {
        let temp_dir = TempDir::new().unwrap();
        let mock_command = if cfg!(windows) {
            temp_dir.path().join("uvm-test.exe")
        } else {
            temp_dir.path().join("uvm-test")
        };
        fs::write(&mock_command, "dummy content").unwrap();
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&mock_command, fs::Permissions::from_mode(0o755)).unwrap(); 
        }

        let old_path = env::var("PATH").unwrap_or_default();
        env::set_var(
            "PATH",
            format!("{};{}", temp_dir.path().display(), old_path),
        );

        let command = ExternalCommand::new(vec!["test".to_string()]);
        let result = command.exec();
        assert!(result.is_err()); // Will error because it's not a real executable

        env::set_var("PATH", old_path);
    }

    #[test]
    fn test_sub_command_path_in_path() {
        let temp_dir = TempDir::new().unwrap();
        let mock_command = if cfg!(windows) {
            temp_dir.path().join("uvm-test.exe")
        } else {
            temp_dir.path().join("uvm-test")
        };
        fs::write(&mock_command, "dummy content").unwrap();

        let old_path = env::var("PATH").unwrap_or_default();
        let path_delimiter = if cfg!(windows) { ";" } else { ":" };
        env::set_var(
            "PATH",
            format!(
                "{}{}{}",
                temp_dir.path().display(),
                path_delimiter,
                old_path
            ),
        );

        let result = sub_command_path("test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), mock_command);

        env::set_var("PATH", old_path);
    }
}
