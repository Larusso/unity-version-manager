#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate serde;
extern crate uvm_cli;
extern crate uvm_core;
extern crate indicatif;

#[macro_use]
extern crate log;
extern crate cluFlock;

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
use uvm_core::install;
use uvm_core::install::InstallVariant;
use uvm_core::unity::hub;
use uvm_core::unity::hub::editors::{EditorInstallation, Editors};
use cluFlock::Flock;
use uvm_core::unity::hub::paths;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Options {
    arg_version: Version,
    arg_destination: Option<PathBuf>,
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

    fn destination(&self) -> &Option<PathBuf> {
        &self.arg_destination
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

type EditorInstallLock = Mutex<Option<io::Result<()>>>;

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

    fn set_editor_install_lock(editor_installed_lock:&Arc<(EditorInstallLock,Condvar)>, value:io::Result<()>) {
        let &(ref lock, ref cvar) = &**editor_installed_lock;
        let mut is_installed = lock.lock().unwrap();
        *is_installed = Some(value);
        cvar.notify_all();
    }

    fn install(install_object:InstallObject, pb:ProgressBar, editor_installed_lock:Arc<(EditorInstallLock,Condvar)>) -> io::Result<()> {
        pb.set_message(&format!("[{}] {}",&install_object.variant, style("download installer").dim()));
        let installer = install::download_installer(install_object.variant.clone(), &install_object.version)
        .map_err(|error| {
            debug!("error loading installer: {}", style(&error).red());
            pb.finish_with_message(&format!("[{}] {}", &install_object.variant, style("error").red().bold()));
            io::Error::new(io::ErrorKind::Other, format!("Failed to fetch installer url \n{}", error.to_string()))
        })?;

        debug!("installer location: {}", &installer.display());

        if install_object.variant != InstallVariant::Editor {
            debug!("aquire editor install lock for {}", &install_object.variant);
            let &(ref lock, ref cvar) = &*editor_installed_lock;
            let mut is_installed = lock.lock().unwrap();
            // As long as the value inside the `Mutex` is false, we wait.
            while (*is_installed).is_none() {
                pb.set_message(&format!("[{}] {}", &install_object.variant, style("waiting").dim()));
                debug!("waiting for editor to finish installation of {}", &install_object.variant);
                is_installed = cvar.wait(is_installed).unwrap();
            }

            if let Some(ref is_installed ) = *is_installed {
                if let Err(err) = is_installed {
                    debug!("editor installation error. Abort installation of {}", &install_object.variant);
                    pb.finish_with_message(&format!("[{}] {}", &install_object.variant, style("editor failed").red().bold()));
                    return Err(io::Error::new(io::ErrorKind::Other, format!("{} failed because of {}", &install_object.variant, InstallVariant::Editor)));
                }
                trace!("editor installation finished. Continue installtion of {}", &install_object.variant);
            }
        }

        let destination = install_object.clone().destination.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Missing installtion destination")
        })?;

        pb.set_message(&format!("[{}] {}", &install_object.variant, style("installing").dim()));
        debug!("install {} to {}",&install_object.variant, &destination.display());
        let install_f = match &install_object.variant {
            InstallVariant::Editor => install::install_editor,
                                 _ => install::install_module,
        };

        install_f(&installer, &destination)
        .map(|result| {
            debug!("installation finished {}.", &install_object.variant);
            pb.finish_with_message(&format!("[{}] {}", &install_object.variant, style("done").green().bold()));
            if install_object.variant == InstallVariant::Editor {
                UvmCommand::set_editor_install_lock(&editor_installed_lock, Ok(()));
            }
            result
        })
        .map_err(|error| {
            debug!("failed to install {}. Error: {}", &install_object.variant, style(&error).red());
            pb.finish_with_message(&format!("[{}] {}", &install_object.variant, style("failed").red().bold()));
            if install_object.variant == InstallVariant::Editor {
                let error = io::Error::new(io::ErrorKind::Other, "failed to install edit");
                UvmCommand::set_editor_install_lock(&editor_installed_lock, Err(error));
            }
            error
        })
    }

    pub fn exec(&self, options:Options) -> io::Result<()> {
        let version = options.version();
        self.stderr.write_line(&format!("{}: {}", style("install unity version").green(), version.to_string())).ok();
        let locks_dir = paths::locks_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Unable to locate locks directory."))?;
        fs::DirBuilder::new().recursive(true).create(&locks_dir)?;

        let lock_file_name = format!("{}.lock", &version);
        let lock_file = locks_dir.join(lock_file_name);
        let lock_file = fs::File::create(lock_file)?;
        let _lock = match lock_file.try_exclusive_lock() {
            Ok(Some(lock)) => {
                trace!("aquire lock for install operation");
                Ok(lock)
            },
            Ok(None) => {
                debug!("install already in progress.");
                debug!("wait for other process to finish.");
                let lock = lock_file.exclusive_lock()?;
                Ok(lock)
            },
            Err(err) => Err(err)
        }?;

        let mut editorInstallation:Option<EditorInstallation> = None;
        let base_dir = if let Some(ref destination) = options.destination() {
            if destination.exists() && !destination.is_dir() {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Requested destination is not a directory."))
            }
            editorInstallation = Some(EditorInstallation::new(options.version().to_owned(), destination.to_path_buf()));
            destination.to_path_buf()
        } else {
            hub::paths::install_path()
            .map(|path| path.join(format!("{}", options.version())))
            .unwrap_or_else(|| Path::new(&format!("/Applications/Unity-{}", options.version())).to_path_buf())
        };

        let installation = Installation::new(base_dir.clone());
        let mut to_install:HashSet<InstallObject> = HashSet::new();
        let mut installed:HashSet<InstallObject> = HashSet::new();

        if installation.is_err() {
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
        let editor_installed_lock = Arc::new((Mutex::new(None), Condvar::new()));
        let mut editor_installing = false;
        let size = to_install.len();
        let mut counter = 1;

        for install_object in diff {
            let pb = multiProgress.add(ProgressBar::new(1));
            pb.set_style(sty.clone());
            pb.enable_steady_tick(100);
            pb.tick();
            pb.set_prefix(&format!("{}/{}", counter, size));
            let editor_installed_lock_c = editor_installed_lock.clone();
            editor_installing |= install_object.variant == InstallVariant::Editor;
            counter+=1;
            threads.push(
                thread::spawn(move || {
                    UvmCommand::install(install_object, pb, editor_installed_lock_c)
                })
            );
        }

        if !editor_installing {
            UvmCommand::set_editor_install_lock(&editor_installed_lock, Ok(()));
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
        })?;

        //write new unity hub editor installation
        if let Some(installation) = editorInstallation {
            let mut editors = Editors::load()
            .and_then(|mut editors| {
                editors.add(installation);
                editors.flush();
                Ok(())
            });
        }

        Ok(())
    }
}
