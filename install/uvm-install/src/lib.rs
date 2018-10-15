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
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use uvm_cli::ColorOption;
use uvm_core::brew;
use uvm_core::unity::Installation;
use uvm_core::unity::Version;
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

        let mut to_install:HashSet<(InstallVariant,Version)> = HashSet::new();
        let mut installed:HashSet<(InstallVariant,Version)> = HashSet::new();

        if let Some(variants) = options.install_variants() {
            for variant in variants {
                to_install.insert((variant, options.version().to_owned()));
            }
        } else {
           info!("No components requested to install");
        }

        if installation.is_err() {
            to_install.insert((InstallVariant::Editor, options.version().to_owned()));

        } else {
            let installation = installation.unwrap();
            info!("Editor already installed at {}", &installation.path().display());

            if !to_install.is_empty() {
                for component in installation.installed_components() {
                    installed.insert((component.into(), options.version().to_owned()));
                }
            }
        }

        if to_install.is_empty() {
            self.stderr.write_line(&format!("{}", style("Nothing to install").green())).ok();
            return Ok(())
        }

        if log_enabled!(log::Level::Info) {
            info!("{}", style("Components to install:").green().bold());
            for c in &to_install {
                info!("{}", style(&c.0).yellow());
            }

            info!("{}", style("Components already installed:").green().bold());
            for c in &installed {
                info!("{}", style(&c.0).yellow());
            }

            let mut intersection = to_install.intersection(&installed).peekable();
            if let Some(_) = intersection.peek() {
                info!("{}", style("Skip variants already installed:").green().bold());
                for c in intersection {
                    info!("{}", style(&c.0).yellow());
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

        let mut threads:Vec<thread::JoinHandle<io::Result<(InstallVariant, PathBuf)>>> = Vec::new();
        let editor_installed_lock = Arc::new((Mutex::new(false), Condvar::new()));
        let mut editor_installing = false;
        for varient_combination in diff {
            let pb = multiProgress.add(ProgressBar::new(1));
            pb.set_style(sty.clone());
            pb.enable_steady_tick(100);
            pb.tick();
            pb.set_prefix(&format!("{}", varient_combination.0));
            let editor_installed_lock_c = editor_installed_lock.clone();

            threads.push(match &varient_combination.0 {
                InstallVariant::Editor => {
                    editor_installing = true;
                    thread::spawn(move || {
                        pb.set_message("download");
                        let installer = uvm_install_core::download_installer(varient_combination.0.clone(), &varient_combination.1)
                        .map_err(|error| {
                            debug!("error loading installer: {}", style(&error).red());
                            pb.finish_with_message(&format!("{}", style("error").red().bold()));
                            error
                        })
                        .map(|installer_path| {
                            debug!("installer location: {}", installer_path.display());
                            (varient_combination.0, installer_path)
                        });

                        pb.set_message("installing");
                        thread::sleep(Duration::from_millis(10000));
                        

                        let &(ref lock, ref cvar) = &*editor_installed_lock_c;
                        let mut is_installed = lock.lock().unwrap();
                        *is_installed = true;
                        // We notify the condvar that the value has changed.
                        cvar.notify_all();
                        pb.finish_with_message("done");
                        return installer
                    })
                },
                _ => {
                    thread::spawn(move || {
                        pb.set_message("download");
                        let installer = uvm_install_core::download_installer(varient_combination.0.clone(), &varient_combination.1)
                        .map_err(|error| {
                            debug!("error loading installer: {}", style(&error).red());
                            pb.finish_with_message(&format!("{}", style("error").red().bold()));
                            error
                        })
                        .map(|installer_path| {
                            debug!("installer location: {}", installer_path.display());
                            (varient_combination.0, installer_path)
                        });

                        {
                            let &(ref lock, ref cvar) = &*editor_installed_lock_c;
                            let mut is_installed = lock.lock().unwrap();
                            // As long as the value inside the `Mutex` is false, we wait.
                            while !*is_installed {
                                pb.set_message("waiting to install");
                                is_installed = cvar.wait(is_installed).unwrap();
                            }
                        }

                        pb.set_message("installing");
                        thread::sleep(Duration::from_millis(10000));
                        pb.finish_with_message("done");
                        return installer
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

        //collect installer paths
        let installer:Vec<thread::Result<io::Result<(InstallVariant, PathBuf)>>> = threads.into_iter().map(thread::JoinHandle::join).collect();

        if installer.into_iter().any(|tr| (tr.is_err() || tr.unwrap().is_err())) {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to download all installer"));
        }

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
