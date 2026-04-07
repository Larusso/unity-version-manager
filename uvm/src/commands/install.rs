use clap::Args;
use console::style;
use indicatif::{HumanBytes, HumanDuration};
use std::io;
use std::path::PathBuf;
use std::time::Instant;
use unity_version::Version;
use uvm_install::{InstallArchitecture, InstallOptions};

use crate::commands::progress::{
    is_interactive, ArcProgressCoordinator, SimpleProgressHandler,
};
use crate::commands::Command;

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Module to install
    ///
    /// A support module to install. You can list all available
    /// modules for a given version using `uvm modules`
    #[arg(short, long = "module", number_of_values = 1)]
    pub modules: Option<Vec<String>>,

    /// Install also synced modules
    ///
    /// Synced modules are optional dependencies of some Unity modules.
    /// e.g. Android SDK for the android module.
    #[arg(long = "with-sync")]
    pub sync: bool,

    /// The architecture to install
    #[arg(long, value_enum, default_value_t = InstallArchitecture::default())]
    pub architecture: InstallArchitecture,

    /// The Unity version to install in the form of `2018.1.0f3`
    pub editor_version: Version,

    /// A directory to install the requested version to
    pub destination: Option<PathBuf>,
}

impl Command for InstallArgs {
    fn execute(&self) -> io::Result<i32> {
        let start_time = Instant::now();
        let version = &self.editor_version;
        let modules = &self.modules;
        let install_sync = self.sync;
        let destination = &self.destination;

        let mut options = InstallOptions::new(version.to_owned())
            .with_install_sync(install_sync)
            .with_architecture(self.architecture);

        if let Some(modules) = modules {
            options = options.with_requested_modules(modules);
        }

        if let Some(destination) = destination {
            options = options.with_destination(destination);
        }

        // Detect interactive mode and create appropriate progress handler
        let interactive = is_interactive();
        let progress_mode = crate::commands::progress::get_progress_mode();

        let coordinator_opt = if interactive {
            // Create a multi-progress coordinator for component installation hierarchy
            // We start with 0 components - the library will update the count after building the graph
            use crate::commands::progress::MultiProgressCoordinator;
            use std::sync::Arc;
            let coordinator = Arc::new(MultiProgressCoordinator::new(0));
            options = options.with_progress_handler(ArcProgressCoordinator(coordinator.clone()));
            Some(coordinator)
        } else if progress_mode != crate::commands::progress::ProgressMode::Disabled {
            // Non-interactive mode - use simple milestone messages (unless --no-progress)
            let simple_handler = SimpleProgressHandler::new("Unity".to_string());
            options = options.with_progress_handler(simple_handler);
            None
        } else {
            // --no-progress: no handler at all
            None
        };

        match options.install() {
            Ok(installation) => {
                let elapsed = start_time.elapsed();

                // Clear progress bars before showing summary
                if let Some(ref coordinator) = coordinator_opt {
                    coordinator.clear();
                }

                // Show installation summary
                if let Some(ref coordinator) = coordinator_opt {
                    let components = coordinator.components_installed();
                    let bytes = coordinator.bytes_downloaded();
                    eprintln!(
                        "\n{} {} ({}) in {}",
                        style("Installed").green().bold(),
                        if components == 1 { "1 component".to_string() } else { format!("{} components", components) },
                        HumanBytes(bytes),
                        HumanDuration(elapsed),
                    );
                } else {
                    eprintln!(
                        "\n{} in {}",
                        style("Installed").green().bold(),
                        HumanDuration(elapsed),
                    );
                }
                eprintln!(
                    "  Unity {} → {}",
                    installation.version(),
                    installation.path().display(),
                );

                Ok(0)
            }
            Err(e) => {
                // Clear progress display before showing error
                if let Some(ref coordinator) = coordinator_opt {
                    coordinator.clear();
                }

                eprintln!("{}: {}", style("Error").red().bold(), e);
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Installation failed: {}", e),
                ))
            }
        }
    }
}
