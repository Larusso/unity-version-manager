use crate::commands::detect::DetectError::{FailedWhileReadingFileSystem, InvalidProjectPath};
use crate::commands::error::CommandError;
use crate::commands::Command;
use err_code::ErrorCode;
use clap::Args;
use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, io};
use console::style;
use thiserror::Error;
use unity_version::error::VersionError;
use unity_version::Version;

#[derive(Args, Debug)]
pub struct DetectCommand {
    pub project_path: Option<PathBuf>,

    #[arg(short, long)]
    pub recursive: bool,
}

#[derive(ErrorCode, Error, Debug)]
pub enum DetectError {
    #[error("Path '{0}' is not a Unity project")]
    #[error_code(1000)]
    NotAUnityProject(PathBuf),

    #[error("Path '{0}' must point to a directory")]
    #[error_code(1001)]
    NotADirectory(PathBuf),

    #[error("Failed to read file system")]
    #[error_code(1002)]
    FailedWhileReadingFileSystem(#[source] io::Error),

    #[error("Failed to parse project version file")]
    #[error_code(1003)]
    FailedToParseProjectVersion(#[source] io::Error),

    #[error("No EditorVersionWithRevision or EditorVersion defined in {0}")]
    #[error_code(1004)]
    NoEditorVersion(PathBuf),

    #[error("Failed to parse unity version string {0}")]
    #[error_code(1005)]
    VersionError(String, #[source] VersionError),

    #[error("Invalid project path")]
    #[error_code(1006)]
    InvalidProjectPath(#[source] io::Error),
}

impl Command for DetectCommand {
    fn execute(&self) -> crate::commands::Result<()> {
        let version = self.detect_version().map_err(|err|
            CommandError::new(
                "Detect unity version in project failed".to_string(),
                err.error_code(), err.into(),
            )
        )?;
        println!("{}", style(version).green().bold());
        Ok(())
    }
}

impl DetectCommand {
    fn detect_version(&self) -> Result<Version, DetectError> {
        let project_path = match self.project_path.as_ref() {
            Some(p) => p,
            _ => &env::current_dir().map_err(|err| InvalidProjectPath(err))?,
        };

        info!(
            "Detect the project version at the path {}",
            project_path.display()
        );
        self.detect_project_version(project_path, self.recursive)
    }

    fn get_project_version<P: AsRef<Path>>(&self, base_dir: P) -> Result<PathBuf, DetectError> {
        let project_version = base_dir
            .as_ref()
            .join("ProjectSettings")
            .join("ProjectVersion.txt");
        if project_version.exists() {
            Ok(project_version)
        } else {
            Err(DetectError::NotAUnityProject(
                base_dir.as_ref().to_path_buf(),
            ))
        }
    }

    pub fn detect_unity_project_dir(
        &self,
        dir: &Path,
        recur: bool,
    ) -> Result<PathBuf, DetectError> {
        if dir.is_dir() {
            if self.get_project_version(dir).is_ok() {
                return Ok(dir.to_path_buf());
            } else if !recur {
                return Err(DetectError::NotAUnityProject(dir.to_path_buf()));
            }

            for entry in fs::read_dir(dir).map_err(|err| FailedWhileReadingFileSystem(err))? {
                let entry = entry.map_err(|err| FailedWhileReadingFileSystem(err))?;
                let path = entry.path();
                if path.is_dir() {
                    let f = self.detect_unity_project_dir(&path, true);
                    if f.is_ok() {
                        return f;
                    }
                }
            }
        }
        Err(DetectError::NotADirectory(dir.to_path_buf()))
    }

    fn detect_project_version(
        &self,
        project_path: &Path,
        recur: bool,
    ) -> Result<Version, DetectError> {
        let project_version = self
            .detect_unity_project_dir(project_path, recur)
            .and_then(|p| self.get_project_version(p))?;

        let file =
            File::open(&project_version).map_err(DetectError::FailedToParseProjectVersion)?;
        let lines = BufReader::new(file).lines();

        let mut editor_versions: HashMap<&'static str, String> = HashMap::with_capacity(2);

        for line in lines {
            if let Ok(line) = line {
                if line.starts_with("m_EditorVersion: ") {
                    let value = line.replace("m_EditorVersion: ", "").trim().to_string();
                    editor_versions.insert("EditorVersion", value);
                }

                if line.starts_with("m_EditorVersionWithRevision: ") {
                    let value = line
                        .replace("m_EditorVersionWithRevision: ", "")
                        .trim()
                        .to_string();
                    editor_versions.insert("EditorVersionWithRevision", value);
                }
            }
        }

        let v = editor_versions
            .get("EditorVersionWithRevision")
            .or_else(|| editor_versions.get("EditorVersion"))
            .ok_or_else(|| DetectError::NoEditorVersion(project_version.to_path_buf()))?;
        Version::from_str(&v).map_err(|err| DetectError::VersionError(v.to_string(), err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn detects_project_version_file() {
        let temp = tempdir().unwrap();
        let project_settings = temp.path().join("ProjectSettings");
        fs::create_dir(&project_settings).unwrap();
        let version_file = project_settings.join("ProjectVersion.txt");

        let version_content = "m_EditorVersion: 2021.3.2f1";
        fs::write(&version_file, version_content).unwrap();

        let cmd = DetectCommand {
            project_path: Some(temp.path().to_path_buf()),
            recursive: false,
        };

        let version = cmd.detect_project_version(temp.path(), false).unwrap();
        assert_eq!(version.to_string(), "2021.3.2f1");
    }

    #[test]
    fn detects_nested_project_when_recursive() {
        let temp = tempdir().unwrap();
        let nested = temp.path().join("nested/project");
        let project_settings = nested.join("ProjectSettings");
        fs::create_dir_all(&project_settings).unwrap();
        let version_file = project_settings.join("ProjectVersion.txt");
        fs::write(&version_file, "m_EditorVersion: 2020.1.5f1").unwrap();

        let cmd = DetectCommand {
            project_path: Some(temp.path().to_path_buf()),
            recursive: true,
        };

        let version = cmd.detect_project_version(temp.path(), true).unwrap();
        assert_eq!(version.to_string(), "2020.1.5f1");
    }

    #[test]
    fn fails_on_missing_project_version() {
        let temp = tempdir().unwrap();

        let cmd = DetectCommand {
            project_path: Some(temp.path().to_path_buf()),
            recursive: false,
        };

        let result = cmd.detect_project_version(temp.path(), false);
        assert!(result.is_err());
    }
}
