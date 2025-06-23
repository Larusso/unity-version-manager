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
use std::env::VarError;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::{fs, io};
use sys::create_installer;
pub use unity_hub::error::UnityError;
pub use unity_hub::error::UnityHubError;
pub use unity_hub::unity;
use unity_hub::unity::hub;
use unity_hub::unity::hub::editors::EditorInstallation;
use unity_hub::unity::hub::module::Module;
use unity_hub::unity::hub::paths;
use unity_hub::unity::hub::paths::locks_dir;
use unity_hub::unity::{Installation, UnityInstallation};
pub use unity_version::error::VersionError;
pub use unity_version::Version;
use uvm_install_graph::{InstallGraph, InstallStatus, UnityComponent, Walker};
pub use uvm_live_platform::error::LivePlatformError;
pub use uvm_live_platform::fetch_release;
use uvm_live_platform::Release;

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
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn ensure_installation_architecture_is_correct<I: Installation>(
    installation: &I,
) -> io::Result<bool> {
    match std::env::var("UVM_ARCHITECTURE_CHECK_ENABLED") {
        Ok(value)
            if value == "1"
                || value == "true"
                || value == "True"
                || value == "TRUE"
                || value == "yes"
                || value == "Yes"
                || value == "YES" =>
        {
            sys::ensure_installation_architecture_is_correct(installation)
        }
        _ => Ok(true),
    }
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
pub fn ensure_installation_architecture_is_correct<I: Installation>(
    installation: &I,
) -> io::Result<bool> {
    Ok(true)
}

pub fn install<V, P, I>(
    version: V,
    mut requested_modules: Option<I>,
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
    let mut additional_modules = vec![];
    let installation = UnityInstallation::new(&base_dir);
    if let Ok(ref installation) = installation {
        info!("Installation found at {}", installation.path().display());
        if ensure_installation_architecture_is_correct(installation)? {
            let modules = installation.installed_modules()?;
            let mut module_ids: HashSet<String> =
                modules.into_iter().map(|m| m.id().to_string()).collect();
            module_ids.insert("Unity".to_string());
            graph.mark_installed(&module_ids);
        } else {
            info!("Architecture mismatch, reinstalling");
            info!("Fetch installed modules:");
            additional_modules = installation
                .installed_modules()?
                .into_iter()
                .map(|m| m.id().to_string())
                .collect();
            // info!("{}", additional_modules.iter().join("\n"));
            fs::remove_dir_all(installation.path())?;
            let version_string =
                format!("{}-{}", unity_release.version, unity_release.short_revision);
            let installer_dir = paths::cache_dir()
                .map(|c| c.join(&format!("installer/{}", version_string)))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "Unable to fetch cache installer directory",
                    )
                })?;
            if installer_dir.exists() {
                info!("Delete installer cache: {}", installer_dir.display());
                fs::remove_dir_all(installer_dir)?;
            }
            info!("Cleanup done");
            graph.mark_all_missing();
        }
    } else {
        info!("\nFresh install");
        graph.mark_all_missing();
    }

    // info!("All available modules for Unity {}", version);
    // print_graph(&graph);
    let additional_modules_iterator = additional_modules.into_iter();
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
            .chain(additional_modules_iterator.map(|c| Ok(c)))
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
            .map(|d| {
                let mut modules = vec![];
                for module in &d.modules {
                    fetch_modules_from_release(&mut modules, module);
                }
                modules
            })
            .unwrap(),
        Ok(m) => m,
    };

    for module in modules.iter_mut() {
        if module.is_installed == false {
            module.is_installed = all_components.contains(module.id());
            trace!("module {} is installed", module.id());
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

fn fetch_modules_from_release(modules: &mut Vec<Module>, module: &uvm_live_platform::Module) {
    modules.push(module.clone().into());
    for sub_module in module.sub_modules() {
        fetch_modules_from_release(modules, sub_module);
    }
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

            info!("install {}", component);
            installer
                .install()
                .map_err(|installer_err| InstallFailed(module.id().to_string(), installer_err))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::cmp::Ordering;
    use std::env;
    use std::fmt::{Display, Formatter};
    use test_binary::build_test_binary;
    use unity_version::ReleaseType;

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct MockInstallation {
        version: Version,
        path: PathBuf,
    }

    impl MockInstallation {
        pub fn new<V: Into<Version>, P: AsRef<Path>>(version: V, path: P) -> Self {
            Self {
                version: version.into(),
                path: path.as_ref().to_path_buf(),
            }
        }
    }

    impl Default for MockInstallation {
        fn default() -> Self {
            Self {
                version: Version::new(6000, 0, 0, ReleaseType::Final, 1),
                path: PathBuf::from("/Applications/Unity/6000.0.0f1"),
            }
        }
    }

    impl Installation for MockInstallation {
        fn path(&self) -> &PathBuf {
            &self.path
        }

        fn version(&self) -> &Version {
            &self.version
        }
    }

    impl Ord for MockInstallation {
        fn cmp(&self, other: &MockInstallation) -> Ordering {
            self.version.cmp(&other.version)
        }
    }

    impl PartialOrd for MockInstallation {
        fn partial_cmp(&self, other: &MockInstallation) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    enum TestArch {
        Arch64,
        X86,
    }

    impl Display for TestArch {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Arch64 => write!(f, "{}", "aarch64"),
                Self::X86 => write!(f, "{}", "x86_64"),
            }
        }
    }

    lazy_static! {
        static ref TEST_UNITY_VERSION_ARM_SUPPORT: Version =
            Version::new(6000, 0, 0, ReleaseType::Final, 1);
        static ref TEST_UNITY_VERSION_NO_ARM_SUPPORT: Version =
            Version::new(2020, 0, 0, ReleaseType::Final, 1);
    }

    #[rstest(
        env_val, test_arch, test_version, expected,
        case::test_arch_check_enabled_with_arm_binary("true", TestArch::Arch64, TEST_UNITY_VERSION_ARM_SUPPORT.clone(), true),
        case::test_arch_check_disabled_with_arm_binary("false", TestArch::Arch64, TEST_UNITY_VERSION_ARM_SUPPORT.clone(), true),
        case::test_arch_check_disabled_with_x86_binary_and_arm_compatible_version_available("false", TestArch::X86, TEST_UNITY_VERSION_ARM_SUPPORT.clone(), true),
        case::test_arch_check_enabled_with_x86_binary_and_arm_compatible_version_not_available("true", TestArch::X86, TEST_UNITY_VERSION_NO_ARM_SUPPORT.clone(), true),
        case::test_arch_check_disabled_with_x86_binary_and_arm_compatible_version_not_available("false", TestArch::X86, TEST_UNITY_VERSION_NO_ARM_SUPPORT.clone(), true),
        case::test_arch_check_enabled_with_x86_binary_and_arm_compatible_version_available("true", TestArch::X86, TEST_UNITY_VERSION_ARM_SUPPORT.clone(), false),
    )]
    #[serial_test::serial]
    fn test_architecture_check(
        env_val: &str,
        test_arch: TestArch,
        test_version: Version,
        expected: bool,
    ) {
        std::env::set_var("UVM_ARCHITECTURE_CHECK_ENABLED", env_val);
        let expected = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            expected
        } else {
            true
        };
    }

    fn run_arch_test(binary_arch: TestArch, unity_version: Version, expected_result: bool) {
        #[cfg(target_os = "macos")]
        const OS_SUFFIX: &str = "apple-darwin";
        #[cfg(target_os = "linux")]
        const OS_SUFFIX: &str = "unknown-linux-gnu";
        #[cfg(target_os = "windows")]
        const OS_SUFFIX: &str = "pc-windows-msvc";

        let test_bin_path =
            build_test_binary("fake-bin", "test-bins").expect("error building test binary");
        let test_bin_path_str = test_bin_path.to_str().unwrap();

        // the test-bins project compiles multiple targets by default
        let aarch_bin_path = test_bin_path_str.replace(
            "target/debug",
            format!("target/{}-{}/debug", binary_arch, OS_SUFFIX).as_str(),
        );

        println!("{}", aarch_bin_path);
        let temp_unity_installation =
            tempfile::tempdir().expect("error creating temporary directory");
        let unity_exec_path = temp_unity_installation
            .path()
            .join("Unity.app/Contents/MacOS/Unity");
        if let Some(parent) = unity_exec_path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directories");
        }
        fs::copy(aarch_bin_path, &unity_exec_path).expect("failed to copy file");
        println!("{}", unity_exec_path.display());

        let installation = MockInstallation::new(unity_version, temp_unity_installation.path());
        assert_eq!(
            ensure_installation_architecture_is_correct(&installation).unwrap(),
            expected_result
        );
    }
}
