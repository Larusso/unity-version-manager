use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::path::{PathBuf,Path};
use std::process;
use std::str::FromStr;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use uvm_core::brew;
use uvm_core::install;
use uvm_core::install::InstallVariant;
use uvm_core::unity::{Installation,Version,Component};

type EditorInstallLock = Mutex<Option<io::Result<()>>>;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct InstallObject {
    version: Version,
    variant: InstallVariant,
    destination: Option<PathBuf>
}

fn set_editor_install_lock(editor_installed_lock:&Arc<(EditorInstallLock,Condvar)>, value:io::Result<()>) {
    let &(ref lock, ref cvar) = &**editor_installed_lock;
    let mut is_installed = lock.lock().unwrap();
    *is_installed = Some(value);
    cvar.notify_all();
}

fn install_component(install_object:InstallObject, editor_installed_lock:Arc<(EditorInstallLock,Condvar)>) -> io::Result<()> {
    let installer = install::download_installer(install_object.variant.clone(), &install_object.version)?;
    debug!("installer location: {}", &installer.display());

    if install_object.variant != InstallVariant::Editor {
        debug!("aquire editor install lock for {}", &install_object.variant);
        let &(ref lock, ref cvar) = &*editor_installed_lock;
        let mut is_installed = lock.lock().unwrap();
        // As long as the value inside the `Mutex` is false, we wait.
        while (*is_installed).is_none() {
            debug!("waiting for editor to finish installation of {}", &install_object.variant);
            is_installed = cvar.wait(is_installed).unwrap();
        }

        if let Some(ref is_installed ) = *is_installed {
            if let Err(err) = is_installed {
                debug!("editor installation error. Abort installation of {}", &install_object.variant);
                return Err(io::Error::new(io::ErrorKind::Other, format!("{} failed because of {}", &install_object.variant, InstallVariant::Editor)));
            }
            trace!("editor installation finished. Continue installtion of {}", &install_object.variant);
        }
    }

    let destination = install_object.clone().destination.ok_or_else(|| {
        io::Error::new(io::ErrorKind::Other, "Missing installtion destination")
    })?;

    debug!("install {} to {}",&install_object.variant, &destination.display());
    let install_f = match &install_object.variant {
        InstallVariant::Editor => install::install_editor,
                             _ => install::install_module,
    };

    install_f(&installer, &destination)
    .map(|result| {
        debug!("installation finished {}.", &install_object.variant);
        if install_object.variant == InstallVariant::Editor {
            set_editor_install_lock(&editor_installed_lock, Ok(()));
        }
        result
    })
    .map_err(|error| {
        if install_object.variant == InstallVariant::Editor {
            let error = io::Error::new(io::ErrorKind::Other, "failed to install edit");
            set_editor_install_lock(&editor_installed_lock, Err(error));
        }
        error
    })
}

pub fn install(version:Version, destination: Option<PathBuf>, variants: Option<HashSet<InstallVariant>>) -> io::Result<()> {
    install::ensure_tap_for_version(&version)?;
    let base_dir = if let Some(ref destination) = destination {
        if destination.exists() && !destination.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Requested destination is not a directory.",
            ));
        }
        destination.to_path_buf()
    } else {
        Path::new(&format!("/Applications/Unity-{}", &version)).to_path_buf()
    };

    let installation = Installation::new(base_dir.clone());
    let mut to_install: HashSet<InstallObject> = HashSet::new();
    let mut installed: HashSet<InstallObject> = HashSet::new();

    if installation.is_err() {
        let installation_data = InstallObject {
            version: version.clone(),
            variant: InstallVariant::Editor,
            destination: Some(base_dir.to_path_buf()),
        };
        to_install.insert(installation_data);

        if let Some(variants) = variants {
            for variant in variants {
                let component: Component = variant.into();
                let variant_destination = component.installpath();
                let installation_data = InstallObject {
                    version: version.clone(),
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
        info!(
            "Editor already installed at {}",
            &installation.path().display()
        );
        let base_dir = installation.path();
        if let Some(variants) = variants {
            for variant in variants {
                let component: Component = variant.into();
                let variant_destination = component.installpath();
                let installation_data = InstallObject {
                    version: version.clone(),
                    variant: component.into(),
                    destination: variant_destination.map(|d| base_dir.join(d)),
                };
                to_install.insert(installation_data);
            }
        }

        if !to_install.is_empty() {
            for component in installation.installed_components() {
                let variant_destination = component.installpath();
                let installation_data = InstallObject {
                    version: version.clone(),
                    variant: component.into(),
                    destination: variant_destination.map(|d| base_dir.join(d)),
                };
                installed.insert(installation_data);
            }
        }
    }

    let mut diff = to_install.difference(&installed).cloned();

    let mut threads: Vec<thread::JoinHandle<io::Result<()>>> = Vec::new();
    let editor_installed_lock = Arc::new((Mutex::new(None), Condvar::new()));
    let mut editor_installing = false;
    let size = to_install.len();
    let mut counter = 1;

    for install_object in diff {
        let editor_installed_lock_c = editor_installed_lock.clone();
        editor_installing |= install_object.variant == InstallVariant::Editor;
        counter += 1;
        threads.push(thread::spawn(move || {
            install_component(install_object, editor_installed_lock_c)
        }));
    }

    if !editor_installing {
        set_editor_install_lock(&editor_installed_lock, Ok(()));
    }

    threads
        .into_iter()
        .map(thread::JoinHandle::join)
        .map(|thread_result| match thread_result {
            Ok(x) => x,
            Err(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Install thread failed",
            )),
        })
        .fold(Ok(()), |acc, r| {
            if let Err(x) = r {
                if let Err(y) = acc {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("{}\n{}", y, x),
                    ));
                }
                return Err(io::Error::new(io::ErrorKind::Other, x));
            }
            acc
        })
}
