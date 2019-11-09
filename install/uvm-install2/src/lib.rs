use self::error::{Error, Result, ResultExt};
use console::style;
use log::*;
use std::collections::HashSet;
use std::io;
use std::path::Path;
pub use uvm_core::error as uvm_core_error;
use uvm_core::unity::hub::editors::{EditorInstallation, Editors};
use uvm_core::unity::{hub, Component, Installation, Manifest, Version};
pub use uvm_core::*;
use uvm_install_core::create_installer;
use uvm_install_graph::{InstallGraph, InstallStatus, Walker};
pub mod error;
use uvm_install_core::Loader;

fn print_graph(graph: &InstallGraph) {
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

/// Installs Unity Editor with optional modules to a destination.
///
/// ```no_run
/// use uvm_install2::unity::{Component, Version};
/// let version = Version::b(2019, 3, 0, 8);
/// let components = [Component::Android, Component::Ios];
/// let install_sync_modules = true;
/// let installation = uvm_install2::install(&version, Some(&components), install_sync_modules, Some("/install/path"))?;
/// # Ok::<(), uvm_install2::error::Error>(())
/// ```
pub fn install<V, P, I>(
    version: V,
    requested_modules: Option<I>,
    install_sync: bool,
    destination: Option<P>,
) -> Result<Installation>
where
    V: AsRef<Version>,
    P: AsRef<Path>,
    I: IntoIterator,
    I::Item: AsRef<Component>,
{
    let version = version.as_ref();
    let mut manifest = Manifest::load(version)?;
    let mut graph = InstallGraph::from(&manifest);

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

    let installation = Installation::new(&base_dir);

    if let Ok(ref installation) = installation {
        let mut installed_components: HashSet<Component> =
            installation.installed_components().collect();
        installed_components.insert(Component::Editor);
        graph.mark_installed(&installed_components);
    } else {
        info!("\nFresh install");
        graph.mark_all_missing();
    }

    info!("All available modules for Unity {}", version);
    print_graph(&graph);
    let base_iterator = [Component::Editor].iter().copied();
    let all_components: HashSet<Component> = match requested_modules {
        Some(modules) => modules
            .into_iter()
            .flat_map(|component| {
                let component = component.as_ref();
                let node = graph.get_node_id(*component).unwrap();
                let mut out = vec![((*component, InstallStatus::Unknown), node)];
                out.append(&mut graph.get_dependend_modules(node));
                if install_sync {
                    out.append(&mut graph.get_sub_modules(node));
                }
                out
            })
            .map(|((c, _), _)| c)
            .chain(base_iterator)
            .collect(),
        None => base_iterator.collect(),
    };

    debug!("\nAll requested components");
    for c in all_components.iter() {
        debug!("- {}", c);
    }

    graph.keep(&all_components);

    info!("\nInstall Graph");
    print_graph(&graph);

    install_module_and_dependencies(&graph, &base_dir)?;

    manifest.mark_installed_modules(all_components);
    write_modules_json(&manifest, &base_dir)?;

    let installation = installation.or_else(|_| Installation::new(&base_dir))?;

    //write new unity hub editor installation
    if let Some(installation) = editor_installation {
        let mut _editors = Editors::load().and_then(|mut editors| {
            editors.add(&installation);
            editors.flush()?;
            Ok(())
        });
    }

    Ok(installation)
}

fn write_modules_json<P: AsRef<Path>>(manifest: &Manifest, output_path: P) -> io::Result<()> {
    use std::fs::OpenOptions;

    let output_path = output_path.as_ref();
    info!(
        "{}",
        style(format!("write {}", output_path.display())).green()
    );
    let output_path = output_path.join("modules.json");
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_path)?;
    manifest.write_modules_json(&mut f)?;
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
