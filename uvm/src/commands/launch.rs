use clap::{Args, ValueEnum};
use console::style;
use log::info;
use std::env;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use unity_hub::unity::{find_installation, list_all_installations, UnityInstallation};
use uvm_detect::detect_project_version;
use uvm_detect::DetectOptions;

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
        let project_path = self
            .project_path
            .as_ref()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| env::current_dir().expect("current working directory"));
        let project_path = DetectOptions::new()
            .recursive(self.recursive)
            .detect_unity_project_dir(&project_path)?;

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
            let version = detect_project_version(project_path)?;
            let installation = find_installation(&version).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to find installation for version {}: {}", version, e),
                )
            })?;
            return Ok(installation);
        }

        // For current installation, we'll use the first available installation
        // This is a simplified approach - in a real implementation you might want to
        // check for a specific "current" installation marker
        let installations = list_all_installations().map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to list installations: {}", e),
            )
        })?;

        let mut installations = installations;
        let installation = installations.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No Unity installations found")
        })?;

        Ok(installation)
    }
}
