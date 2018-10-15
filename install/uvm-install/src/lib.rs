#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate serde;
extern crate uvm_cli;
extern crate uvm_core;
extern crate uvm_install_core;
extern crate indicatif;

#[macro_use]
extern crate log;

use console::style;
use console::Term;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::path::{PathBuf,Path};
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use uvm_cli::ColorOption;
use uvm_core::brew;
use uvm_core::unity::{Installation,Version,Component};
use uvm_install_core::InstallVariant;

#[derive(Debug, Deserialize)]
pub struct Options {
    #[serde(with = "uvm_core::unity::unity_version_format")]
    arg_version: Version,
    flag_verbose: bool,
    flag_debug: bool,
    flag_android: bool,
    flag_ios: bool,
    flag_webgl: bool,
    flag_mobile: bool,
    flag_linux: bool,
    flag_windows: bool,
    flag_desktop: bool,
    flag_all: bool,
    flag_color: ColorOption,
}

impl Options {
    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn install_variants(&self) -> Option<HashSet<InstallVariant>> {
        if self.flag_android || self.flag_ios || self.flag_webgl || self.flag_linux
            || self.flag_windows || self.flag_mobile || self.flag_desktop || self.flag_all
        {
            let mut variants: HashSet<InstallVariant> = HashSet::with_capacity(5);

            if self.flag_android || self.flag_mobile || self.flag_all {
                variants.insert(InstallVariant::Android);
            }

            if self.flag_ios || self.flag_mobile || self.flag_all {
                variants.insert(InstallVariant::Ios);
            }

            if self.flag_webgl || self.flag_mobile || self.flag_all {
                variants.insert(InstallVariant::WebGl);
            }

            let check_version = Version::from_str("2018.0.0b0").unwrap();
            if (self.flag_windows || self.flag_desktop || self.flag_all) && self.version() >= &check_version {
                variants.insert(InstallVariant::WindowsMono);
            }

            if (self.flag_windows || self.flag_desktop || self.flag_all) && self.version() < &check_version {
                variants.insert(InstallVariant::Windows);
            }

            if self.flag_linux || self.flag_desktop || self.flag_all {
                variants.insert(InstallVariant::Linux);
            }
            return Some(variants);
        }
        None
    }
}

impl uvm_cli::Options for Options {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn debug(&self) -> bool {
        self.flag_debug
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct InstallObject {
    version: Version,
    variant: InstallVariant,
    destination: Option<PathBuf>
}

pub struct UvmCommand {
    stdout: Term,
    stderr: Term
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    fn progress_draw_target<T>(options:&T) -> ProgressDrawTarget
    where
        T: uvm_cli::Options,
    {
        if( options.debug()) {
            ProgressDrawTarget::hidden()
        } else {
            ProgressDrawTarget::stderr()
        }
    }

    pub fn exec(&self, options:Options) -> io::Result<()> {
        self.stderr.write_line(&format!("{}: {}", style("install unity version").green(), options.version().to_string())).ok();

        uvm_install_core::ensure_tap_for_version(&options.version())?;
        let installation = uvm_core::find_installation(&options.version());

        let mut to_install:HashSet<InstallObject> = HashSet::new();
        let mut installed:HashSet<InstallObject> = HashSet::new();

        if installation.is_err() {
            let base_dir = Path::new(&format!("/Applications/Unity-{}", options.version())).to_path_buf();
            let installation_data = InstallObject {
                            version: options.version().to_owned(),
                            variant: InstallVariant::Editor,
                            destination: Some(base_dir.to_path_buf()),
                        };
            to_install.insert(installation_data);

            if let Some(variants) = options.install_variants() {
                for variant in variants {
                    let component:Component = variant.into();
                    let variant_destination = component.installpath();
                    let installation_data = InstallObject {
                                    version: options.version().to_owned(),
                                    variant: component.into(),
                                    destination: variant_destination.map(|d| base_dir.join(d)),
                                };
                    to_install.insert(installation_data);
                }
            } else {
               info!("No components requested to install");
            }
        } else {
            let installation = installation.unwrap();
            info!("Editor already installed at {}", &installation.path().display());
            let base_dir = installation.path();
            if let Some(variants) = options.install_variants() {
                for variant in variants {
                    let component:Component = variant.into();
                    let variant_destination = component.installpath();
                    let installation_data = InstallObject {
                                    version: options.version().to_owned(),
                                    variant: component.into(),
                                    destination: variant_destination.map(|d| base_dir.join(d))
                                };
                    to_install.insert(installation_data);
                }
            }

            if !to_install.is_empty() {
                for component in installation.installed_components() {
                    let variant_destination = component.installpath();
                    let installation_data = InstallObject {
                                    version: options.version().to_owned(),
                                    variant: component.into(),
                                    destination: variant_destination.map(|d| base_dir.join(d))
                                };
                    installed.insert(installation_data);
                }
            }
        }

        if to_install.is_empty() {
            self.stderr.write_line(&format!("{}", style("Nothing to install").green())).ok();
            return Ok(())
        }

        if log_enabled!(log::Level::Info) {
            if !to_install.is_empty() {
                info!("{}", style("Components to install:").green().bold());
                for c in &to_install {
                    info!("{}", style(&c.variant).yellow());
                }
            }

            if !installed.is_empty() {
                info!("{}", style("Components already installed:").green().bold());
                for c in &installed {
                    info!("{}", style(&c.variant).yellow());
                }
            }

            let mut intersection = to_install.intersection(&installed).peekable();
            if let Some(_) = intersection.peek() {
                info!("{}", style("Skip variants already installed:").green().bold());
                for c in intersection {
                    info!("{}", style(&c.variant).yellow());
                }
            }
        }

        let mut diff = to_install.difference(&installed).cloned().peekable();
        if let None = diff.peek() {
            self.stderr.write_line(&format!("{}", style("Nothing to install").green())).ok();
            return Ok(())
        }

        let multiProgress = MultiProgress::new();
        multiProgress.set_draw_target(UvmCommand::progress_draw_target(&options));
        let sty = ProgressStyle::default_bar()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim>15} {spinner} {wide_msg}");

        self.stderr.write_line("download installer");

        let mut threads:Vec<thread::JoinHandle<io::Result<()>>> = Vec::new();
        let editor_installed_lock = Arc::new((Mutex::new(false), Condvar::new()));
        let mut editor_installing = false;
        for varient_combination in diff {
            let pb = multiProgress.add(ProgressBar::new(1));
            pb.set_style(sty.clone());
            pb.enable_steady_tick(100);
            pb.tick();
            pb.set_prefix(&format!("{}", varient_combination.variant));
            let editor_installed_lock_c = editor_installed_lock.clone();

            threads.push(match &varient_combination.variant {
                InstallVariant::Editor => {
                    editor_installing = true;
                    thread::spawn(move || {
                        pb.set_message("download");
                        let installer = uvm_install_core::download_installer(varient_combination.variant.clone(), &varient_combination.version)
                        .map_err(|error| {
                            debug!("error loading installer: {}", style(&error).red());
                            pb.finish_with_message(&format!("{}", style("error").red().bold()));
                            error
                        });

                        if installer.is_err() {
                            pb.finish_with_message("error downloading installer");
                            return Err(installer.unwrap_err())
                        }
                        let installer = installer.unwrap();
                        debug!("installer location: {}", &installer.display());

                        pb.set_message("installing");
                        let destination = varient_combination.destination.unwrap();
                        debug!("install unity editor to {}", &destination.display());
                        let result = uvm_install_core::installer::install_editor(&installer, &destination);

                        if result.is_err() {
                            pb.finish_with_message("failed to install editor");
                            return Err(result.unwrap_err())
                        }

                        let &(ref lock, ref cvar) = &*editor_installed_lock_c;
                        let mut is_installed = lock.lock().unwrap();
                        *is_installed = true;
                        // We notify the condvar that the value has changed.
                        cvar.notify_all();
                        pb.finish_with_message("done");
                        Ok(())
                    })
                },
                _ => {
                    thread::spawn(move || {
                        pb.set_message("download");
                        let installer = uvm_install_core::download_installer(varient_combination.variant.clone(), &varient_combination.version)
                        .map_err(|error| {
                            debug!("error loading installer: {}", style(&error).red());
                            pb.finish_with_message(&format!("{}", style("error").red().bold()));
                            error
                        });

                        if installer.is_err() {
                            pb.finish_with_message("error downloading installer");
                            return Err(installer.unwrap_err())
                        }

                        let installer = installer.unwrap();
                        debug!("installer location: {}", &installer.display());

                        {
                            let &(ref lock, ref cvar) = &*editor_installed_lock_c;
                            let mut is_installed = lock.lock().unwrap();
                            // As long as the value inside the `Mutex` is false, we wait.
                            while !*is_installed {
                                pb.set_message("waiting to install");
                                is_installed = cvar.wait(is_installed).unwrap();
                            }
                        }

                        if let Some(destination) = varient_combination.destination {
                            pb.set_message("installing");
                            debug!("install unity component {} to {}", &varient_combination.variant, &destination.display());
                            let result = uvm_install_core::installer::install_module(&installer, &destination);
                            if result.is_err() {
                                pb.finish_with_message("failed to install component");
                                let error = result.unwrap_err();
                                error!("{}", error);
                                return Err(error)
                            }

                            pb.finish_with_message("done");
                            return Ok(())
                        } else {
                            pb.finish_with_message("failed to install: missing destination");
                            return Err(io::Error::new(io::ErrorKind::Other, "Missing install destination"))
                        }
                    })
                }
            });
        }

        if !editor_installing {
            let &(ref lock, ref cvar) = &*editor_installed_lock;
            let mut is_installed = lock.lock().unwrap();
            *is_installed = true;
            // We notify the condvar that the value has changed.
            cvar.notify_all();
        }

        //wait for all progress bars to finish
        multiProgress.join();

        // //collect installer paths
        // let installer:Vec<thread::Result<io::Result<(InstallVariant, PathBuf)>>> = threads.into_iter().map(thread::JoinHandle::join).collect();
        //
        // if installer.into_iter().any(|tr| (tr.is_err() || tr.unwrap().is_err())) {
        //     return Err(io::Error::new(io::ErrorKind::Other, "Failed to download all installer"));
        // }

        //debug!("installer {:?}", &installer);

        // debug!("download installer");
        // let installer = uvm_install_core::download_installer(InstallVariant::Editor, &options.version())?;
        // debug!("installer location: {}", installer.display());

        //let mut diff = to_install.difference(&installed).peekable();
        // if let Some(_) = diff.peek() {
        //     let mut child = brew::cask::install(diff)?;
        //     let status = child.wait()?;
        //
        //     if !status.success() {
        //         return Err(io::Error::new(io::ErrorKind::Other, "Failed to install casks"));
        //     }
        // }
        // else {
        //     return Err(io::Error::new(io::ErrorKind::Other, "Version and all support packages already installed"));
        // }

        Ok(())
    }
}
