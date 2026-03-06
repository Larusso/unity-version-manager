use anyhow::Result;
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;
use unity_version::Version;
use uvm_live_platform::{FetchRelease, UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform};

#[derive(Args, Debug)]
pub struct DownloadModulesJsonCommand {
    /// Unity version to fetch modules for (e.g. `2023.1.0f1`)
    version: Version,

    /// Target platform (defaults to current host platform)
    #[arg(long, value_enum, default_value_t = UnityReleaseDownloadPlatform::default())]
    platform: UnityReleaseDownloadPlatform,

    /// Target architecture (defaults to current host architecture)
    #[arg(long, value_enum, default_value_t = UnityReleaseDownloadArchitecture::default())]
    architecture: UnityReleaseDownloadArchitecture,

    /// Write output to a file instead of stdout (parent directories are created if missing)
    #[arg(long, short)]
    output: Option<PathBuf>,
}

impl DownloadModulesJsonCommand {
    pub fn execute(self) -> io::Result<i32> {
        match self.run() {
            Ok(_) => Ok(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                Ok(1)
            }
        }
    }

    fn run(&self) -> Result<()> {
        let release = FetchRelease::builder(self.version.clone())
            .with_extended_lts()
            .with_u7_alpha()
            .with_platform(self.platform)
            .with_architecture(self.architecture)
            .fetch()
            .map_err(|_| anyhow::anyhow!("Version '{}' not found", self.version))?;

        let modules: Vec<_> = release
            .downloads
            .iter()
            .flat_map(|d| d.iter_modules())
            .collect();

        let json = serde_json::to_string_pretty(&modules)?;

        match &self.output {
            Some(path) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, &json)?;
            }
            None => {
                io::stdout().write_all(json.as_bytes())?;
                writeln!(io::stdout())?;
            }
        }

        Ok(())
    }
}
