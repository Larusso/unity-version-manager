use indicatif;
use uvm_cli;
use log::*;
use self::error::{Error, Result};
use console::{style, Term};
use indicatif::{ProgressDrawTarget, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use uvm_cli::options::ColorOption;
use uvm_install_core as install;
use uvm_core::unity::hub;
use uvm_core::unity::hub::editors::{EditorInstallation, Editors};
use uvm_core::unity::hub::paths;
use uvm_core::unity::v2::Manifest;
use uvm_core::unity::{Component, Installation, Version};
use uvm_install_core::create_installer;
use uvm_core::*;

mod progress;
mod error;
use self::progress::{MultiProgress, ProgressBar};

pub trait Options {
    fn debug(&self) -> bool {
        self.verbose()
    }

    fn verbose(&self) -> bool {
        false
    }

    fn color(&self) -> &ColorOption {
        &ColorOption::Auto
    }
}

pub trait InstallerOptions {
    fn version(&self) -> &Version;
    fn install_variants(&self) -> Option<HashSet<Component>>;
    fn destination(&self) -> &Option<PathBuf>;
    fn skip_verification(&self) -> bool;
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct InstallObject {
    version: Version,
    component: Component,
    verify: bool,
    destination: Option<PathBuf>,
}

pub struct UvmCommand {
    stderr: Term,
}

type EditorInstallLock = Mutex<Option<io::Result<()>>>;

impl Default for UvmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stderr: Term::stderr(),
        }
    }

    fn progress_draw_target<T>(options: &T) -> ProgressDrawTarget
    where
        T: Options,
    {
        if options.debug() {
            ProgressDrawTarget::hidden()
        } else {
            ProgressDrawTarget::stderr()
        }
    }

    fn set_editor_install_lock(
        editor_installed_lock: &Arc<(EditorInstallLock, Condvar)>,
        value: io::Result<()>,
    ) {
        let &(ref lock, ref cvar) = &**editor_installed_lock;
        let mut is_installed = lock.lock().unwrap();
        *is_installed = Some(value);
        cvar.notify_all();
    }

    fn install(
        install_object: &InstallObject,
        pb: &ProgressBar,
        editor_installed_lock: Arc<(EditorInstallLock, Condvar)>,
    ) -> Result<()> {
        pb.set_message(&format!("{}", style("download installer").yellow()));
        let manifest = Manifest::load(&install_object.version).map_err(|_|
            io::Error::new(io::ErrorKind::Other, "unable to load manifest")
        )?;

        let mut installer_loader = install::Loader::new(
            install_object.component,
            &manifest,
        );

        let sty = ProgressStyle::default_bar()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .progress_chars("=>-")
            .template("{prefix:<20} {spinner} {msg:<20} [{bar:28.yellow/green}] {percent:>3}%");

        pb.set_style(sty);

        installer_loader.set_progress_handle(pb);
        installer_loader.verify_installer(install_object.verify);

        let installer = installer_loader.download().map_err(|error| {
            debug!("error loading installer: {}", style(&error).red());
            pb.finish_with_message(&format!("{}", style("error").red().bold()));
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to fetch installer url \n{}", error.to_string()),
            )
        })?;
        let sty = ProgressStyle::default_bar()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:<20} {spinner} {msg:<20}");
        pb.set_style(sty);

        debug!("installer location: {}", &installer.display());

        if install_object.component != Component::Editor {
            debug!("aquire editor install lock for {}", &install_object.component);
            let &(ref lock, ref cvar) = &*editor_installed_lock;
            let mut is_installed = lock.lock().unwrap();
            // As long as the value inside the `Mutex` is false, we wait.
            while (*is_installed).is_none() {
                pb.set_message(&format!("{}", style("waiting").white().dim()));
                debug!(
                    "waiting for editor to finish installation of {}",
                    &install_object.component
                );
                is_installed = cvar.wait(is_installed).unwrap();
            }

            if let Some(ref is_installed) = *is_installed {
                if is_installed.is_err() {
                    debug!(
                        "editor installation error. Abort installation of {}",
                        &install_object.component
                    );
                    pb.finish_with_message(&format!("{}", style("editor failed").red().bold()));
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "{} failed because of {}",
                            &install_object.component,
                            Component::Editor
                        ),
                    ).into());
                }
                trace!(
                    "editor installation finished. Continue installtion of {}",
                    &install_object.component
                );
            }
        }

        info!("create installer for {}", &install_object.component);
        let module = manifest.get(&install_object.component).unwrap();
        let installer = create_installer(&install_object.destination.clone().unwrap(), installer, module)?;

        pb.set_message(&format!("{}", style("installing").yellow()));

        installer.install()
            .map(|result| {
                debug!("installation finished {}.", &install_object.component);
                pb.finish_with_message(&format!("{}", style("done").green().bold()));
                if install_object.component == Component::Editor {
                    UvmCommand::set_editor_install_lock(&editor_installed_lock, Ok(()));
                }
                result
            })
            .map_err(|error| {
                debug!(
                    "failed to install {}. Error: {}",
                    &install_object.component,
                    style(&error).red()
                );
                pb.finish_with_message(&format!("{}", style("failed").red().bold()));
                if install_object.component == Component::Editor {
                    let error = io::Error::new(io::ErrorKind::Other, "failed to install edit");
                    UvmCommand::set_editor_install_lock(&editor_installed_lock, Err(error));
                }
                error.into()
            })
    }

    pub fn exec<O>(&self, options: &O) -> Result<()> where O: InstallerOptions + Options {
        let version = options.version();
        self.stderr
            .write_line(&format!(
                "{}: {}",
                style("install api version").green(),
                version.to_string()
            ))
            .ok();
        
        let version_string = format!("{}-{}", version, version.version_hash()?);
        let locks_dir = paths::locks_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Unable to locate locks directory.")
        })?;

        fs::DirBuilder::new().recursive(true).create(&locks_dir)?;
        lock_process!(locks_dir.join(format!("{}.lock", &version_string)));

        let mut editor_installation: Option<EditorInstallation> = None;
        let base_dir = if let Some(ref destination) = options.destination() {
            if destination.exists() && !destination.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Requested destination is not a directory.",
                ).into());
            }
            editor_installation = Some(EditorInstallation::new(
                options.version().to_owned(),
                destination.to_path_buf(),
            ));
            destination.to_path_buf()
        } else {
            hub::paths::install_path()
                .map(|path| path.join(format!("{}", options.version())))
                .unwrap_or_else(|| {
                    Path::new(&format!("/Applications/Unity-{}", options.version())).to_path_buf()
                })
        };

        let installation = Installation::new(base_dir.clone());
        let mut to_install: HashSet<InstallObject> = HashSet::new();
        let mut installed: HashSet<InstallObject> = HashSet::new();

        if installation.is_err() {
            let installation_data = InstallObject {
                version: options.version().to_owned(),
                component: Component::Editor,
                destination: Some(base_dir.to_path_buf()),
                verify: !options.skip_verification(),
            };
            to_install.insert(installation_data);

            if let Some(components) = options.install_variants() {
                for component in components {
                    //fix better
                    let variant_destination = if cfg![windows] {
                        Some(base_dir.to_path_buf())
                    } else {
                        component.installpath()
                    };
                    let installation_data = InstallObject {
                        version: options.version().to_owned(),
                        component: component,
                        destination: variant_destination.map(|d| base_dir.join(d)),
                        verify: !options.skip_verification(),
                    };
                    to_install.insert(installation_data);
                }
            } else {
                info!("No components requested to install");
            }
        } else {
            let installation = installation.unwrap();
            info!(
                "Editor already installed at {}",
                &installation.path().display()
            );
            let base_dir = installation.path();
            if let Some(components) = options.install_variants() {
                for component in components {
                    //fix better
                    let variant_destination = if cfg![windows] {
                        Some(base_dir.to_path_buf())
                    } else {
                        component.installpath()
                    };
                    let installation_data = InstallObject {
                        version: options.version().to_owned(),
                        component: component,
                        destination: variant_destination.map(|d| base_dir.join(d)),
                        verify: !options.skip_verification(),
                    };
                    to_install.insert(installation_data);
                }
            }

            if !to_install.is_empty() {
                for component in installation.installed_components() {
                    let variant_destination = component.installpath();
                    let installation_data = InstallObject {
                        version: options.version().to_owned(),
                        component: component,
                        destination: variant_destination.map(|d| base_dir.join(d)),
                        verify: !options.skip_verification(),
                    };
                    installed.insert(installation_data);
                }
            }
        }

        if to_install.is_empty() {
            self.stderr
                .write_line(&format!("{}", style("Nothing to install").green()))
                .ok();
            return Ok(());
        }

        if log_enabled!(log::Level::Info) {
            if !to_install.is_empty() {
                info!("{}", style("Components to install:").green().bold());
                for c in &to_install {
                    info!("{}", style(&c.component).yellow());
                }
            }

            if !installed.is_empty() {
                info!("{}", style("Components already installed:").green().bold());
                for c in &installed {
                    info!("{}", style(&c.component).yellow());
                }
            }

            let mut intersection = to_install.intersection(&installed).peekable();
            if intersection.peek().is_some() {
                info!(
                    "{}",
                    style("Skip variants already installed:").green().bold()
                );
                for c in intersection {
                    info!("{}", style(&c.component).yellow());
                }
            }
        }

        let mut diff = to_install.difference(&installed).cloned().peekable();
        if diff.peek().is_none() {
            self.stderr
                .write_line(&format!("{}", style("Nothing to install").green()))
                .ok();
            return Ok(());
        }

        let multi_progress = MultiProgress::new();
        multi_progress.set_draw_target(UvmCommand::progress_draw_target(options));
        let sty = ProgressStyle::default_bar()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:<20.bold.dim} {spinner} {msg:<20.green}");

        let mut threads: Vec<thread::JoinHandle<Result<()>>> = Vec::new();
        let editor_installed_lock = Arc::new((Mutex::new(None), Condvar::new()));
        let mut editor_installing = false;
        let size = to_install.len();
        let mut counter = 1;

        for install_object in diff {
            let pb = multi_progress.add(indicatif::ProgressBar::new(1));
            pb.set_style(sty.clone());
            pb.enable_steady_tick(100);
            pb.set_draw_delta(0);
            pb.tick();
            pb.set_prefix(&format!(
                "{}/{} [{}]",
                counter, size, install_object.component
            ));
            let editor_installed_lock_c = editor_installed_lock.clone();
            editor_installing |= install_object.component == Component::Editor;
            counter += 1;
            threads.push(thread::spawn(move || {
                UvmCommand::install(&install_object, &pb, editor_installed_lock_c)
            }));
        }

        if !editor_installing {
            UvmCommand::set_editor_install_lock(&editor_installed_lock, Ok(()));
        }

        //wait for all progress bars to finish
        multi_progress.join_and_clear()?;
        threads
            .into_iter()
            .map(thread::JoinHandle::join)
            .map(|thread_result| match thread_result {
                Ok(x) => x,
                Err(_) => Err(Error::from("Install thread failed")),
            })
            .fold(Ok(()), |acc, r| {
                if let Err(x) = r {
                    if let Err(y) = acc {
                        return Err(Error::from(format!("{}\n{}", y, x)));
                    }
                    return Err(x);
                }
                acc
            })?;

        //write new api hub editor installation
        if let Some(installation) = editor_installation {
            let mut _editors = Editors::load().and_then(|mut editors| {
                editors.add(&installation);
                editors.flush()?;
                Ok(())
            });
        }

        Ok(())
    }
}
