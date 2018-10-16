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

    fn install(install_object:InstallObject, pb:ProgressBar, editor_installed_lock:Arc<(Mutex<bool>,Condvar)>) -> io::Result<()> {
        pb.set_message("download installer");
        let installer = uvm_install_core::download_installer(install_object.variant.clone(), &install_object.version)
        .map_err(|error| {
            debug!("error loading installer: {}", style(&error).red());
            pb.finish_with_message(&format!("{}", style("error").red().bold()));
            error
        })?;

        debug!("installer location: {}", &installer.display());

        if install_object.variant != InstallVariant::Editor {
            debug!("aquire editor install lock for {}", &install_object.variant);
            let &(ref lock, ref cvar) = &*editor_installed_lock;
            let mut is_installed = lock.lock().unwrap();
            // As long as the value inside the `Mutex` is false, we wait.
            debug!("editor is installed: {}", *is_installed);
            while !*is_installed {
                pb.set_message("waiting for editor to finish installation");
                debug!("waiting for editor to finish installation {}", &install_object.variant);
                is_installed = cvar.wait(is_installed).unwrap();
            }
            debug!("editor installation finish {}", &install_object.variant);
        }

        let destination = install_object.clone().destination.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Missing installtion destination")
        })?;

        pb.set_message("installing");
        debug!("install {} to {}",&install_object.variant, &destination.display());
        let install_f = match &install_object.variant {
            InstallVariant::Editor => uvm_install_core::installer::install_editor,
                                 _ => uvm_install_core::installer::install_module,
        };

        install_f(&installer, &destination)
        .map_err(|error| {
            debug!("failed to install {}. Error: {}", &install_object.variant, style(&error).red());
            pb.finish_with_message(&format!("{}", style("failed to install").red().bold()));
            error
        })?;

        if install_object.variant == InstallVariant::Editor {
            let &(ref lock, ref cvar) = &*editor_installed_lock;
            let mut is_installed = lock.lock().unwrap();
            *is_installed = true;
            // We notify the condvar that the value has changed.
            cvar.notify_all();
        }
        pb.finish_with_message("done");
        Ok(())
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

        let mut threads:Vec<thread::JoinHandle<io::Result<()>>> = Vec::new();
        let editor_installed_lock = Arc::new((Mutex::new(false), Condvar::new()));
        let mut editor_installing = false;
        for install_object in diff {
            let pb = multiProgress.add(ProgressBar::new(1));
            pb.set_style(sty.clone());
            pb.enable_steady_tick(100);
            pb.tick();
            pb.set_prefix(&format!("{}", install_object.variant));
            let editor_installed_lock_c = editor_installed_lock.clone();
            editor_installing |= install_object.variant == InstallVariant::Editor;
            threads.push(
                thread::spawn(move || {
                    UvmCommand::install(install_object, pb, editor_installed_lock_c)
                })
            );
        }

        if !editor_installing {
            let &(ref lock, ref cvar) = &*editor_installed_lock;
            let mut is_installed = lock.lock().unwrap();
            *is_installed = true;
            // We notify the condvar that the value has changed.
            cvar.notify_all();
        }

        //wait for all progress bars to finish
        multiProgress.join_and_clear();
        threads.into_iter()
        .map(thread::JoinHandle::join)
        .map(|thread_result| {
            match thread_result {
                Ok(x) => x,
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Install thread failed"))
            }
        })
        .fold(Ok(()), |acc, r| {
            if let Err(x) = r {
                if let Err(y) = acc {
                    return Err(io::Error::new(io::ErrorKind::Other, format!("{}\n{}", y, x)))
                }
                return Err(io::Error::new(io::ErrorKind::Other, x))
            }
            acc
        })
    }
}
