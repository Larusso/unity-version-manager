mod error;

use cluFlock::{FlockLock, SharedFlock};
use error::*;
use log::{debug, info, trace};
use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};
use unity_hub::unity::hub::editors::EditorInstallation;
use unity_hub::unity::UnityInstallation;
use unity_hub::unity::{hub, Installation};
use unity_version::Version;
use uvm_install_graph::{InstallGraph, InstallStatus, UnityComponent, Walker};
use uvm_paths::locks_dir;

#[macro_export]
#[cfg(unix)]
macro_rules! lock_process {
    ($lock_path:expr) => {
        let lock_file = fs::File::create($lock_path)?;
        let _lock = lock_process_or_wait(&lock_file)?;
    };
}

#[cfg(unix)]
pub fn lock_process_or_wait<'a>(lock_file: &'a File) -> io::Result<FlockLock<&'a File>> {
    match lock_file.try_lock() {
        Ok(lock) => {
            trace!("aquired process lock.");
            Ok(lock)
        }
        Err(_) => {
            debug!("progress lock already aquired.");
            debug!("wait for other process to finish.");
            let lock = lock_file.wait_lock()?;
            Ok(lock)
        } //Err(err) => Err(err),
    }
}

#[cfg(windows)]
pub fn lock_process_or_wait(_: &File) -> io::Result<()> {
    Ok(())
}

fn print_graph<'a>(graph: &'a InstallGraph<'a>) {
    use console::Style;

    for node in graph.topo().iter(graph.context()) {
        let component = graph.component(node).unwrap();
        let install_status = graph.install_status(node).unwrap();
        let prefix: String = [' '].iter().cycle().take(graph.depth(node) * 2).collect();

        let style = match install_status {
            InstallStatus::Unknown => Style::default().dim(),
            InstallStatus::Missing => Style::default().yellow().blink(),
            InstallStatus::Installed => Style::default().green(),
        };

        info!(
            "{}- {} ({})",
            prefix,
            component,
            style.apply_to(install_status)
        );
    }
}

#[macro_export]
#[cfg(windows)]
macro_rules! lock_process {
    ($lock_path:expr) => {};
}

pub fn install<V, P, I>(
    version: V,
    requested_modules: Option<I>,
    install_sync: bool,
    destination: Option<P>,
) -> error::Result<()>
where
    V: AsRef<Version>,
    P: AsRef<Path>,
    I: IntoIterator,
    I::Item: Into<String>,
{
    let version = version.as_ref();
    let version_string = version.to_string();

    let locks_dir = locks_dir().ok_or_else(|| {
        InstallError::LockProcessFailure(io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to locate locks directory.",
        ))
    })?;

    fs::DirBuilder::new().recursive(true).create(&locks_dir)?;
    lock_process!(locks_dir.join(format!("{}.lock", version_string)));

    let unity_release = uvm_live_platform::fetch_release(version.to_owned())?;
    eprintln!("{:#?}", unity_release);
    let mut graph = InstallGraph::from(&unity_release);

    //

    let mut editor_installation: Option<EditorInstallation> = None;
    let base_dir = if let Some(destination) = destination {
        let destination = destination.as_ref();
        if destination.exists() && !destination.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Requested destination is not a directory.",
            )
            .into());
        }

        editor_installation = Some(EditorInstallation::new(
            version.to_owned(),
            destination.to_path_buf(),
        ));
        destination.to_path_buf()
    } else {
        hub::paths::install_path()
            .map(|path| path.join(format!("{}", version)))
            .or_else(|| {
                {
                    #[cfg(any(target_os = "windows", target_os = "macos"))]
                    let application_path = dirs_2::application_dir();
                    #[cfg(target_os = "linux")]
                    let application_path = dirs_2::executable_dir();
                    application_path
                }
                .map(|path| path.join(format!("Unity-{}", version)))
            })
            .expect("default installation directory")
    };

    let installation = UnityInstallation::new(&base_dir);
    if let Ok(ref installation) = installation {
        let modules = installation.installed_modules()?;
        let mut module_ids: HashSet<String> = modules.into_iter().map(|m| m.id).collect();
        module_ids.insert("Unity".to_string());
        graph.mark_installed(&module_ids);
    } else {
        info!("\nFresh install");
        graph.mark_all_missing();
    }

    // info!("All available modules for Unity {}", version);
    // print_graph(&graph);

    let base_iterator = ["Unity".to_string()].into_iter();
    let all_components: HashSet<String> = match requested_modules {
        Some(modules) => modules
            .into_iter()
            .flat_map(|module| {
                let module = module.into();
                let node = graph.get_node_id(module.clone()).ok_or_else(|| {
                    debug!(
                        "Unsupported module '{}' for selected api version {}",
                        module, version
                    );
                    InstallError::UnsupportedModule(module.to_string(), version.to_string())
                });

                match node {
                    Ok(node) => {
                        let mut out = vec![Ok(module.to_string())];
                        out.append(
                            &mut graph
                                .get_dependend_modules(node)
                                .iter()
                                .map({
                                    |((c, _), _)| match c {
                                        UnityComponent::Editor(_) => Ok("Unity".to_string()),
                                        UnityComponent::Module(m) => Ok(m.id.to_string()),
                                    }
                                })
                                .collect(),
                        );
                        if install_sync {
                            out.append(
                                &mut graph
                                    .get_sub_modules(node)
                                    .iter()
                                    .map({
                                        |((c, _), _)| match c {
                                            UnityComponent::Editor(_) => Ok("Unity".to_string()),
                                            UnityComponent::Module(m) => Ok(m.id.to_string()),
                                        }
                                    })
                                    .collect(),
                            );
                        }
                        out
                    }
                    Err(err) => vec![Err(err.into())],
                }
            })
            .chain(base_iterator.map(|c| Ok(c)))
            .collect::<Result<HashSet<_>>>(),
        None => base_iterator.map(|c| Ok(c)).collect::<Result<HashSet<_>>>(),
    }?;

    debug!("\nAll requested components");
    for c in all_components.iter() {
        debug!("- {}", c);
    }

    graph.keep(&all_components);

    info!("\nInstall Graph");
    print_graph(&graph);

    Ok(())
}

fn install_module_and_dependencies<P: AsRef<Path>>(
    graph: &InstallGraph,
    base_dir: P,
) -> Result<()> {
    let base_dir = base_dir.as_ref();
    for node in graph.topo().iter(graph.context()) {
        if let Some(InstallStatus::Missing) = graph.install_status(node) {
            let component = graph.component(node).unwrap();
            info!("install {}", component);
            info!("download installer for {}", component);
            let loader = Loader::new(*component, graph.manifest());
            let installer = loader.download()?;

            info!("create installer for {}", component);
            let module = graph.manifest().get(&component).unwrap();
            let installer = create_installer(base_dir, installer, module)?;
            info!("install");
            installer
                .install()
                .chain_err(|| Error::from(format!("failed to install {}", component)))?;
        }
    }

    Ok(())
}
