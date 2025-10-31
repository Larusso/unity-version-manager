use std::path::PathBuf;
use std::io;
use clap::Args;
use console::style;
use unity_version::Version;
use uvm_install::{InstallArchitecture, InstallOptions};

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
        let version = &self.editor_version;
        let modules = &self.modules;
        let install_sync = self.sync;
        let destination = &self.destination;

        eprintln!(
            "Request to install Unity Editor version {} with modules {:?} to destination: {:?}",
            version, modules, destination
        );

        let mut options = InstallOptions::new(version.to_owned())
            .with_install_sync(install_sync)
            .with_architecture(self.architecture);

        if let Some(modules) = modules {
            options = options.with_requested_modules(modules);
        }

        if let Some(destination) = destination {
            options = options.with_destination(destination);
        }

        match options.install() {
            Ok(installation) => {
                eprintln!(
                    "{}: Unity {} installed at {}",
                    style("Finish").green().bold(),
                    installation.version(),
                    installation.path().display()
                );
                Ok(0)
            }
            Err(e) => {
                eprintln!("{}: {}", style("Error").red().bold(), e);
                Err(io::Error::new(io::ErrorKind::Other, format!("Installation failed: {}", e)))
            }
        }
    }
}
