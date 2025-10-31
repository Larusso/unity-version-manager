use clap::Args;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::fs::remove_dir_all;
use std::io;
use std::path::PathBuf;
use console::style;
use unity_hub::unity::{find_installation, UnityInstallation, Installation};
use unity_hub::unity::hub::module::Module;
use unity_version::Version;

use crate::commands::Command;

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// The unity version to uninstall (defaults to removing the entire editor)
    pub version: Version,

    /// Module IDs to uninstall (can be repeated). If specified, only these modules will be removed instead of the entire editor
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub module: Vec<String>,

    /// Uninstall all removable modules instead of the entire editor
    #[arg(short, long)]
    pub all: bool,
}

impl Command for UninstallArgs {
    fn execute(&self) -> io::Result<i32> {
        info!("Uninstalling Unity version: {}", self.version);

        // Find the Unity installation
        let installation = find_installation(&self.version)
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Unable to find installation for version {}: {}", self.version, e)))?;

        // Load modules from the installation's modules.json file
        let mut all_modules = installation.get_modules()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to read modules.json: {}", e)))?;

        // Determine what to uninstall
        if self.all {
            // If --all is specified, try to uninstall all installed modules except core ones
            info!("Checking {} modules for --all uninstall", all_modules.len());
            for module in &all_modules {
                debug!("Module: {} | Category: {} | Destination: {:?} | Installed: {} | ID: {}", 
                    module.base.description(), 
                    module.base.category(), 
                    module.base.destination(),
                    module.is_installed,
                    module.id()
                );
            }
            
            let to_uninstall: Vec<_> = all_modules.iter()
                .filter(|module| module.is_installed && self.can_uninstall_module(module, &installation))
                .collect();

            if to_uninstall.is_empty() {
                eprintln!("{}", style("No removable modules found").yellow());
                return Ok(0);
            }

            eprintln!(
                "{}: {}",
                style("uninstall unity modules").green(),
                &self.version
            );
            eprintln!("{}", style("Modules to uninstall:").green());

            let mut uninstalled_count = 0;
            let mut uninstalled_ids: Vec<String> = Vec::new();
            
            for module in &to_uninstall {
                if let Some(module_path) = self.get_module_install_path(module, &installation) {
                    if module_path.exists() {
                        eprintln!("{}: {} ({})", 
                            style("Remove").cyan(), 
                            style(module.id()).cyan().bold(),
                            style(module.base.description()).dim()
                        );
                        remove_dir_all(&module_path)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to remove {}: {}", module_path.display(), e)))?;
                        uninstalled_count += 1;
                        uninstalled_ids.push(module.id().to_string());
                    } else {
                        debug!("Module '{}' path does not exist: {}", module.id(), module_path.display());
                    }
                } else {
                    debug!("Could not determine install path for module '{}'", module.id());
                }
            }

            if uninstalled_count > 0 {
                // Mark uninstalled modules as not installed
                for module in &mut all_modules {
                    if uninstalled_ids.contains(&module.id().to_string()) {
                        debug!("Marking module '{}' as not installed in modules.json", module.id());
                        module.is_installed = false;
                    }
                }
                
                // Update modules.json with the changes
                if let Err(e) = installation.write_modules(all_modules) {
                    warn!("Failed to update modules.json: {}", e);
                }
                eprintln!("{}: {} modules uninstalled", style("Finish").green().bold(), uninstalled_count);
            } else {
                eprintln!("{}", style("No modules were uninstalled").yellow());
            }
        } else if !self.module.is_empty() {
            // If specific modules are requested, find them
            let requested_modules: HashSet<String> = self.module.iter().cloned().collect();
            let to_uninstall: Vec<_> = all_modules.iter()
                .filter(|module| module.is_installed && requested_modules.contains(module.id()))
                .collect();

            if to_uninstall.is_empty() {
                eprintln!("{}", style("No matching modules found").yellow());
                return Ok(1);
            }

            eprintln!(
                "{}: {}",
                style("uninstall unity modules").green(),
                &self.version
            );
            eprintln!("{}", style("Modules to uninstall:").green());

            let mut uninstalled_count = 0;
            let mut uninstalled_ids: Vec<String> = Vec::new();
            
            for module in &to_uninstall {
                if !self.can_uninstall_module(module, &installation) {
                    warn!("Skipping module '{}' ({}): Cannot be uninstalled", module.id(), module.base.description());
                    eprintln!("{}: {} ({})", 
                        style("Skip").yellow(), 
                        style(module.id()).yellow().bold(),
                        style("cannot be uninstalled").dim()
                    );
                    continue;
                }

                if let Some(module_path) = self.get_module_install_path(module, &installation) {
                    if module_path.exists() {
                        eprintln!("{}: {} ({})", 
                            style("Remove").cyan(), 
                            style(module.id()).cyan().bold(),
                            style(module.base.description()).dim()
                        );
                        remove_dir_all(&module_path)
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to remove {}: {}", module_path.display(), e)))?;
                        uninstalled_count += 1;
                        uninstalled_ids.push(module.id().to_string());
                    } else {
                        debug!("Module '{}' path does not exist: {}", module.id(), module_path.display());
                    }
                } else {
                    debug!("Could not determine install path for module '{}'", module.id());
                }
            }

            if uninstalled_count > 0 {
                // Mark uninstalled modules as not installed
                for module in &mut all_modules {
                    if uninstalled_ids.contains(&module.id().to_string()) {
                        debug!("Marking module '{}' as not installed in modules.json", module.id());
                        module.is_installed = false;
                    }
                }
                
                // Update modules.json with the changes
                if let Err(e) = installation.write_modules(all_modules) {
                    warn!("Failed to update modules.json: {}", e);
                }
                eprintln!("{}: {} modules uninstalled", style("Finish").green().bold(), uninstalled_count);
            } else {
                eprintln!("{}", style("No modules were uninstalled").yellow());
            }
        } else {
            // Default behavior: uninstall the entire Unity editor
            debug!("Removing Unity editor at {}", installation.path().display());
            eprintln!(
                "{}: {}",
                style("uninstall unity editor").green(),
                &self.version
            );
            remove_dir_all(installation.path())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to remove Unity installation at {}: {}", installation.path().display(), e)))?;
            eprintln!("{}", style("Unity editor uninstalled").green().bold());
        }
        Ok(0)
    }
}

impl UninstallArgs {
    fn can_uninstall_module(&self, module: &Module, installation: &UnityInstallation) -> bool {
        // Skip modules without a destination
        let _destination = match module.base.destination() {
            Some(dest) if !dest.is_empty() => dest,
            _ => {
                debug!("Skipping module '{}': no destination", module.id());
                return false;
            }
        };

        let module_path = self.get_module_install_path(module, installation);
        let module_path = match module_path {
            Some(path) => path,
            None => {
                debug!("Skipping module '{}': could not determine install path", module.id());
                return false;
            }
        };

        let install_path = installation.path();

        // Rule 1: Module path must be INSIDE the Unity installation directory
        if !module_path.starts_with(install_path) {
            debug!("Skipping module '{}': path '{}' is outside Unity installation '{}'", 
                module.id(), module_path.display(), install_path.display());
            return false;
        }

        // Rule 2: Module path must NOT be the same as the Unity installation directory
        // (this would delete the entire Unity installation)
        if module_path == *install_path {
            debug!("Skipping module '{}': path '{}' would delete entire Unity installation", 
                module.id(), module_path.display());
            return false;
        }

        // Rule 3: Resolve any symbolic links and check again
        let canonical_module_path = module_path.canonicalize().unwrap_or(module_path.clone());
        let canonical_install_path = install_path.canonicalize().unwrap_or(install_path.clone());
        
        if !canonical_module_path.starts_with(&canonical_install_path) {
            debug!("Skipping module '{}': canonical path '{}' is outside Unity installation '{}'", 
                module.id(), canonical_module_path.display(), canonical_install_path.display());
            return false;
        }

        if canonical_module_path == canonical_install_path {
            debug!("Skipping module '{}': canonical path '{}' would delete entire Unity installation", 
                module.id(), canonical_module_path.display());
            return false;
        }

        debug!("Module '{}' is safe to uninstall: '{}' is a subdirectory of '{}'", 
            module.id(), module_path.display(), install_path.display());
        true
    }

    fn get_module_install_path(&self, module: &Module, installation: &UnityInstallation) -> Option<PathBuf> {
        if let Some(destination) = module.base.destination() {
            if destination.is_empty() {
                return None;
            }
            
            // Replace {UNITY_PATH} placeholder with the actual installation path
            let resolved_destination = if destination.contains("{UNITY_PATH}") {
                destination.replace("{UNITY_PATH}", installation.path().to_string_lossy().as_ref())
            } else {
                // If no placeholder, treat destination as relative to installation path
                return Some(installation.path().join(destination));
            };
            
            Some(PathBuf::from(resolved_destination))
        } else {
            None
        }
    }

}
