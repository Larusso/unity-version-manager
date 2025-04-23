mod error;
mod install;
mod sys;
use crate::error::InstallError::{InstallFailed, InstallerCreatedFailed, LoadingInstallerFailed};
pub use error::*;
use install::utils;
use install::{InstallManifest, Loader};
use lazy_static::lazy_static;
use log::{debug, info, trace};
use ssri::Integrity;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::{fs, io};
use sys::create_installer;
use unity_hub::unity::hub::editors::EditorInstallation;
use unity_hub::unity::hub::paths::locks_dir;
use unity_hub::unity::hub;
pub use unity_hub::unity;
pub use unity_hub::error::UnityError;
pub use unity_hub::error::UnityHubError;
pub use unity_version::error::VersionError;
use unity_hub::unity::{UnityInstallation, Installation};
pub use unity_version::Version;
use uvm_install_graph::{InstallGraph, InstallStatus, UnityComponent, Walker};
pub use uvm_live_platform::fetch_release;
pub use uvm_live_platform::error::LivePlatformError;

lazy_static! {
    static ref UNITY_BASE_PATTERN: &'static Path = Path::new("{UNITY_PATH}");
}

impl AsRef<Path> for UNITY_BASE_PATTERN {
    fn as_ref(&self) -> &Path {
        self.deref()
    }
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

pub fn install<V, P, I>(
    version: V,
    requested_modules: Option<I>,
    install_sync: bool,
    destination: Option<P>,
) -> Result<UnityInstallation>
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

    let unity_release = fetch_release(version.to_owned())?;
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
        let mut module_ids: HashSet<String> =
            modules.into_iter().map(|m| m.id().to_string()).collect();
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
                let node = graph.get_node_id(&module).ok_or_else(|| {
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
                                        UnityComponent::Module(m) => Ok(m.id().to_string()),
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
                                            UnityComponent::Module(m) => Ok(m.id().to_string()),
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

    install_module_and_dependencies(&graph, &base_dir)?;
    let installation = installation.or_else(|_| UnityInstallation::new(&base_dir))?;
    let mut modules = match installation.get_modules() {
        Err(_) => unity_release
            .downloads
            .first()
            .cloned()
            .map(|d| d.modules.into_iter().map(|m| m.into()).collect())
            .unwrap(),
        Ok(m) => m,
    };

    for module in modules.iter_mut() {
        if module.is_installed == false {
            module.is_installed = all_components.contains(module.id())
        }
    }

    write_modules_json(&installation, modules)?;

    //write new api hub editor installation
    if let Some(installation) = editor_installation {
        let mut _editors = unity_hub::Editors::load().and_then(|mut editors| {
            editors.add(&installation);
            editors.flush()?;
            Ok(())
        });
    }

    Ok(installation)
}

fn write_modules_json(
    installation: &UnityInstallation,
    modules: Vec<unity_hub::unity::hub::module::Module>,
) -> io::Result<()> {
    use console::style;
    use std::fs::OpenOptions;
    use std::io::Write;

    let output_path = installation
        .location()
        .parent()
        .unwrap()
        .join("modules.json");
    info!(
        "{}",
        style(format!("write {}", output_path.display())).green()
    );
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_path)?;

    let j = serde_json::to_string_pretty(&modules)?;
    write!(f, "{}", j)?;
    trace!("{}", j);
    Ok(())
}

struct UnityComponent2<'a>(UnityComponent<'a>);

impl<'a> Deref for UnityComponent2<'a> {
    type Target = UnityComponent<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for UnityComponent2<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> InstallManifest for UnityComponent2<'a> {
    fn is_editor(&self) -> bool {
        match self.0 {
            UnityComponent::Editor(_) => true,
            _ => false,
        }
    }
    fn id(&self) -> &str {
        match self.0 {
            UnityComponent::Editor(_) => "Unity",
            UnityComponent::Module(m) => m.id(),
        }
    }
    fn install_size(&self) -> u64 {
        let download_size = match self.0 {
            UnityComponent::Editor(e) => e.download_size,
            UnityComponent::Module(m) => m.download_size,
        };
        download_size.to_bytes() as u64
    }

    fn download_url(&self) -> &str {
        match self.0 {
            UnityComponent::Editor(e) => &e.release_file.url,
            UnityComponent::Module(m) => &m.release_file().url,
        }
    }

    //TODO find a way without clone
    fn integrity(&self) -> Option<Integrity> {
        match self.0 {
            UnityComponent::Editor(e) => e.release_file.integrity.clone(),
            UnityComponent::Module(m) => m.release_file().integrity.clone(),
        }
    }

    fn install_rename_from_to<P: AsRef<Path>>(&self, base_path: P) -> Option<(PathBuf, PathBuf)> {
        match self.0 {
            UnityComponent::Editor(_) => None,
            UnityComponent::Module(m) => {
                if let Some(extracted_path_rename) = &m.extracted_path_rename() {
                    Some((
                        strip_unity_base_url(&extracted_path_rename.from, &base_path),
                        strip_unity_base_url(&extracted_path_rename.to, &base_path),
                    ))
                } else {
                    None
                }
            }
        }
    }

    fn install_destination<P: AsRef<Path>>(&self, base_path: P) -> Option<PathBuf> {
        match self.0 {
            UnityComponent::Editor(_) => Some(base_path.as_ref().to_path_buf()),
            UnityComponent::Module(m) => {
                if let Some(destination) = &m.destination() {
                    Some(strip_unity_base_url(destination, &base_path))
                } else {
                    None
                }
            }
        }
    }
}

fn strip_unity_base_url<P: AsRef<Path>, Q: AsRef<Path>>(path: P, base_dir: Q) -> PathBuf {
    let path = path.as_ref();
    base_dir
        .as_ref()
        .join(&path.strip_prefix(&UNITY_BASE_PATTERN).unwrap_or(path))
}

fn install_module_and_dependencies<'a, P: AsRef<Path>>(
    graph: &'a InstallGraph<'a>,
    base_dir: P,
) -> Result<()> {
    let base_dir = base_dir.as_ref();
    for node in graph.topo().iter(graph.context()) {
        if let Some(InstallStatus::Missing) = graph.install_status(node) {
            let component = graph.component(node).unwrap();
            let module = UnityComponent2(component);
            let version = &graph.release().version;
            let hash = &graph.release().short_revision;

            info!("install {}", module.id());
            info!("download installer for {}", module.id());

            let loader = Loader::new(version, hash, &module);
            let installer = loader
                .download()
                .map_err(|installer_err| LoadingInstallerFailed(installer_err))?;

            info!("create installer for {}", component);
            let installer = create_installer(base_dir, installer, &module)
                .map_err(|installer_err| InstallerCreatedFailed(installer_err))?;

            info!("install");
            installer
                .install()
                .map_err(|installer_err| InstallFailed(installer_err))?;
        }
    }

    Ok(())
}
