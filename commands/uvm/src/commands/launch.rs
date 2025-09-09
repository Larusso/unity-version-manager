use clap::{Args, ValueEnum};
use log::info;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::fs;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::fmt;
use console::style;
use unity_hub::unity::{find_installation, list_all_installations, UnityInstallation};
use unity_version::Version;

#[derive(Debug, Clone, ValueEnum)]
pub enum UnityPlatform {
    Win32,
    Win64,
    OSX,
    Linux,
    Linux64,
    IOS,
    Android,
    Web,
    WebStreamed,
    WebGl,
    XboxOne,
    PS4,
    PSP2,
    WsaPlayer,
    Tizen,
    SamsungTV,
}

impl fmt::Display for UnityPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnityPlatform::Win32 => write!(f, "Win32"),
            UnityPlatform::Win64 => write!(f, "Win64"),
            UnityPlatform::OSX => write!(f, "OSX"),
            UnityPlatform::Linux => write!(f, "Linux"),
            UnityPlatform::Linux64 => write!(f, "Linux64"),
            UnityPlatform::IOS => write!(f, "IOS"),
            UnityPlatform::Android => write!(f, "Android"),
            UnityPlatform::Web => write!(f, "Web"),
            UnityPlatform::WebStreamed => write!(f, "WebStreamed"),
            UnityPlatform::WebGl => write!(f, "WebGl"),
            UnityPlatform::XboxOne => write!(f, "XboxOne"),
            UnityPlatform::PS4 => write!(f, "PS4"),
            UnityPlatform::PSP2 => write!(f, "PSP2"),
            UnityPlatform::WsaPlayer => write!(f, "WsaPlayer"),
            UnityPlatform::Tizen => write!(f, "Tizen"),
            UnityPlatform::SamsungTV => write!(f, "SamsungTV"),
        }
    }
}

#[derive(Args, Debug)]
pub struct LaunchCommand {
    /// the build platform to open the project with
    #[arg(short, long, value_enum)]
    platform: Option<UnityPlatform>,

    /// Detects a api project recursivly from current working or <project-path> directory.
    #[arg(short, long)]
    recursive: bool,

    /// Will launch try to launch the project with the Unity version the project was created from.
    #[arg(short, long)]
    force_project_version: bool,

    /// Path to the Unity Project
    project_path: Option<PathBuf>,
}

impl LaunchCommand {
    pub fn execute(&self) -> io::Result<i32> {
        let project_path = self.project_path
            .as_ref()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| env::current_dir().expect("current working directory"));
        let project_path = self.detect_unity_project_dir(&project_path, self.recursive)?;

        info!("launch project: {}", style(&project_path.display()).cyan());

        let installation = self.get_installation(&project_path, self.force_project_version)?;

        info!(
            "launch api version: {}",
            style(installation.version().to_string()).cyan()
        );

        let mut command = process::Command::new(
            installation
                .exec_path()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        );

        if let Some(ref platform) = self.platform {
            command.arg("-buildTarget").arg(platform.to_string());
        };

        command
            .arg("-projectPath")
            .arg(project_path.canonicalize().unwrap().to_str().unwrap());

        command.spawn()?;
        Ok(0)
    }

    fn get_installation(
        &self,
        project_path: &Path,
        use_project_version: bool,
    ) -> io::Result<UnityInstallation> {
        if use_project_version {
            let version = self.detect_project_version(project_path)?;
            let installation = find_installation(&version)
                .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to find installation for version {}: {}", version, e)))?;
            return Ok(installation);
        }

        // For current installation, we'll use the first available installation
        // This is a simplified approach - in a real implementation you might want to
        // check for a specific "current" installation marker
        let installations = list_all_installations()
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to list installations: {}", e)))?;
        
        let mut installations = installations;
        let installation = installations
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No Unity installations found"))?;
        
        Ok(installation)
    }

    fn detect_unity_project_dir(&self, dir: &Path, recur: bool) -> io::Result<PathBuf> {
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

    fn detect_project_version(&self, project_path: &Path) -> io::Result<Version> {
        let project_version = self.detect_unity_project_dir(project_path, false)
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
