use clap::Args;
use log::info;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use unity_version::Version;

#[derive(Args, Debug)]
pub struct DetectCommand {
    pub project_path: Option<PathBuf>,

    #[arg(short, long)]
    pub recursive: bool,
}

impl DetectCommand {
    pub fn execute(&self) -> io::Result<i32> {
        let project_path = match self.project_path.as_ref() {
            Some(p) => p,
            _ => &env::current_dir()?,
        };
        
        info!("Detect the project version at path {}", project_path.display());
        let version = self.detect_project_version(project_path, self.recursive)?;
        println!("{}", version);
        Ok(0)
    }

    fn get_project_version<P: AsRef<Path>>(&self, base_dir: P) -> io::Result<PathBuf> {
        let project_version = base_dir
            .as_ref()
            .join("ProjectSettings")
            .join("ProjectVersion.txt");
        if project_version.exists() {
            Ok(project_version)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "directory {} is not a Unity project",
                    base_dir.as_ref().display()
                ),
            ))
        }
    }

    pub fn detect_unity_project_dir(&self, dir: &Path, recur: bool) -> io::Result<PathBuf> {
        let error = Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to find a Unity project",
        ));

        if dir.is_dir() {
            if self.get_project_version(dir).is_ok() {
                return Ok(dir.to_path_buf());
            } else if !recur {
                return error;
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let f = self.detect_unity_project_dir(&path, true);
                    if f.is_ok() {
                        return f;
                    }
                }
            }
        }
        error
    }

    fn detect_project_version(&self, project_path: &Path, recur: bool) -> io::Result<Version> {
        let project_version = self.detect_unity_project_dir(project_path, recur)
            .and_then(|p| self.get_project_version(p))?;

        let file = File::open(project_version)?;
        let lines = BufReader::new(file).lines();

        let mut editor_versions: HashMap<&'static str, String> = HashMap::with_capacity(2);

        for line in lines {
            if let Ok(line) = line {
                if line.starts_with("m_EditorVersion: ") {
                    let value = line.replace("m_EditorVersion: ", "");
                    editor_versions.insert("EditorVersion", value.to_owned());
                }

                if line.starts_with("m_EditorVersionWithRevision: ") {
                    let value = line.replace("m_EditorVersionWithRevision: ", "");
                    editor_versions.insert("EditorVersionWithRevision", value.to_owned());
                }
            }
        }

        let v = editor_versions
            .get("EditorVersionWithRevision")
            .or_else(|| editor_versions.get("EditorVersion"))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Can't parse Unity version")
            })?;
        Version::from_str(&v)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Can't parse Unity version"))
    }
}
