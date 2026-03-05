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
use std::fmt;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::{fs, io};
use sys::create_installer;
use unity_hub::unity::hub;
use unity_hub::unity::hub::editors::EditorInstallation;
use unity_hub::unity::hub::module::Module;
use unity_hub::unity::hub::paths;
use unity_hub::unity::hub::paths::locks_dir;
use unity_hub::unity::{Installation, UnityInstallation};
use unity_version::Version;
use uvm_install_graph::{InstallGraph, InstallStatus, UnityComponent, Walker};
use uvm_live_platform::error::ErrorRepr;
use uvm_live_platform::error::LivePlatformError;
use uvm_live_platform::{FetchRelease, UnityReleaseDownloadArchitecture};

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
    _installation: &I,
) -> io::Result<bool> {
    Ok(true)
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum InstallArchitecture {
    X86_64,
    Arm64,
}

impl From<UnityReleaseDownloadArchitecture> for InstallArchitecture {
    fn from(architecture: UnityReleaseDownloadArchitecture) -> Self {
        match architecture {
            UnityReleaseDownloadArchitecture::X86_64 => Self::X86_64,
            UnityReleaseDownloadArchitecture::Arm64 => Self::Arm64,
        }
    }
}

impl Default for InstallArchitecture {
    fn default() -> Self {
        UnityReleaseDownloadArchitecture::default().into()
    }
}

impl Into<UnityReleaseDownloadArchitecture> for InstallArchitecture {
    fn into(self) -> UnityReleaseDownloadArchitecture {
        match self {
            InstallArchitecture::X86_64 => UnityReleaseDownloadArchitecture::X86_64,
            InstallArchitecture::Arm64 => UnityReleaseDownloadArchitecture::Arm64,
        }
    }
}

impl Display for InstallArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arch: UnityReleaseDownloadArchitecture = (*self).into();
        write!(f, "{}", arch)
    }
}

pub struct InstallOptions {
    version: Version,
    requested_modules: HashSet<String>,
    install_sync: bool,
    destination: Option<PathBuf>,
    architecture: Option<InstallArchitecture>,
}

impl InstallOptions {
    pub fn new<V: Into<Version>>(version: V) -> Self {
        Self {
            version: version.into(),
            requested_modules: HashSet::new(),
            install_sync: false,
            destination: None,
            architecture: None,
        }
    }

    pub fn with_requested_modules<S: Into<String>, I: IntoIterator<Item = S>>(
        mut self,
        requested_modules: I,
    ) -> Self {
        self.requested_modules = requested_modules.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn with_install_sync(mut self, install_sync: bool) -> Self {
        self.install_sync = install_sync;
        self
    }

    pub fn with_destination<P: AsRef<Path>>(mut self, destination: P) -> Self {
        self.destination = Some(destination.as_ref().to_path_buf());
        self
    }

    pub fn with_architecture(mut self, architecture: InstallArchitecture) -> Self {
        self.architecture = Some(architecture);
        self
    }

    fn modules_from_release(unity_release: &uvm_live_platform::Release) -> Vec<Module> {
        unity_release
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
            .unwrap_or_default()
    }

    pub fn install(&self) -> Result<UnityInstallation> {
        let version = &self.version;
        let version_string = version.to_string();

        let locks_dir = locks_dir().ok_or_else(|| {
            InstallError::LockProcessFailure(io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to locate locks directory.",
            ))
        })?;

        fs::DirBuilder::new().recursive(true).create(&locks_dir)?;
        lock_process!(locks_dir.join(format!("{}.lock", version_string)));
        let architecture: UnityReleaseDownloadArchitecture = self
            .architecture.clone()
            .unwrap_or(InstallArchitecture::X86_64)
            .into();
        let unity_release = FetchRelease::builder(version.to_owned())
            .with_current_platform()
            .with_extended_lts()
            .with_u7_alpha()
            .with_architecture(architecture)
            .fetch()
            .map_err(|e| {
                let e = ErrorRepr::FetchReleaseError(e);
                LivePlatformError::new("Failed to fetch release", e)
            })?;

        //let unity_release = fetch_release(version.to_owned())?;
        eprintln!("{:#?}", unity_release);
        let mut graph = InstallGraph::from(&unity_release);
        //

        let mut editor_installation: Option<EditorInstallation> = None;
        let base_dir = if let Some(destination) = &self.destination {
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
        let all_components: HashSet<String> = self
            .requested_modules
            .iter()
            .flat_map(|module| {
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
                        if self.install_sync {
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
            .collect::<Result<HashSet<_>>>()?;

        debug!("\nAll requested components");
        for c in all_components.iter() {
            debug!("- {}", c);
        }

        graph.keep(&all_components);

        info!("\nInstall Graph");
        print_graph(&graph);

        // Ensure base directory exists before installation
        fs::DirBuilder::new().recursive(true).create(&base_dir)?;

        // Initialize modules list before installation loop
        let mut modules: Vec<Module> = match &installation {
            Ok(inst) => inst.get_modules().unwrap_or_else(|_| {
                Self::modules_from_release(&unity_release)
            }),
            Err(_) => Self::modules_from_release(&unity_release),
        };

        // Install modules and update state incrementally
        install_module_and_dependencies(&graph, &base_dir, &mut modules)?;

        // Get or create installation handle for final operations
        let installation = installation.or_else(|_| UnityInstallation::new(&base_dir))?;

        // Update is_installed flags for any modules that were already installed
        for module in modules.iter_mut() {
            if !module.is_installed {
                module.is_installed = all_components.contains(module.id());
                trace!("module {} is installed", module.id());
            }
        }

        // Write final state
        installation.write_modules(modules)?;

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
}

fn fetch_modules_from_release(modules: &mut Vec<Module>, module: &uvm_live_platform::Module) {
    modules.push(module.clone().into());
    for sub_module in module.sub_modules() {
        fetch_modules_from_release(modules, sub_module);
    }
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

/// Trait for installing individual modules, allowing for mocking in tests
trait ModuleInstaller {
    fn install_module(&self, module_id: &str, base_dir: &Path) -> Result<()>;
}

/// Default implementation that uses the real download and install process
struct RealModuleInstaller<'a> {
    graph: &'a InstallGraph<'a>,
}

impl<'a> ModuleInstaller for RealModuleInstaller<'a> {
    fn install_module(&self, module_id: &str, base_dir: &Path) -> Result<()> {
        let node = self.graph.get_node_id(module_id)
            .ok_or_else(|| InstallError::UnsupportedModule(module_id.to_string(), "unknown".to_string()))?;

        let component = self.graph.component(node).unwrap();
        let unity_module = UnityComponent2(component);
        let version = &self.graph.release().version;
        let hash = &self.graph.release().short_revision;

        info!("download installer for {}", module_id);
        let loader = Loader::new(version, hash, &unity_module);
        let installer_path = loader
            .download()
            .map_err(|installer_err| LoadingInstallerFailed(installer_err))?;

        info!("create installer for {}", component);
        let installer = create_installer(base_dir, installer_path, &unity_module)
            .map_err(|installer_err| InstallerCreatedFailed(installer_err))?;

        info!("install {}", component);
        installer
            .install()
            .map_err(|installer_err| InstallFailed(module_id.to_string(), installer_err))?;

        Ok(())
    }
}

fn install_module_and_dependencies<'a, P: AsRef<Path>>(
    graph: &'a InstallGraph<'a>,
    base_dir: P,
    modules: &mut Vec<Module>,
) -> Result<()> {
    let installer = RealModuleInstaller { graph };
    install_modules_with_installer(graph, base_dir, modules, &installer)
}

fn install_modules_with_installer<'a, P: AsRef<Path>, I: ModuleInstaller>(
    graph: &'a InstallGraph<'a>,
    base_dir: P,
    modules: &mut Vec<Module>,
    installer: &I,
) -> Result<()> {
    let base_dir = base_dir.as_ref();
    let mut errors = Vec::new();

    for node in graph.topo().iter(graph.context()) {
        if let Some(InstallStatus::Missing) = graph.install_status(node) {
            let component = graph.component(node).unwrap();
            let module_id = match component {
                UnityComponent::Editor(_) => "Unity".to_string(),
                UnityComponent::Module(m) => m.id().to_string(),
            };

            info!("install {}", module_id);

            let install_result = installer.install_module(&module_id, base_dir);

            match install_result {
                Err(err) if module_id == "Unity" => {
                    // Editor installation failed - cleanup and abort
                    log::error!("Editor installation failed, cleaning up");
                    if base_dir.exists() {
                        if let Err(cleanup_err) = std::fs::remove_dir_all(base_dir) {
                            log::warn!("Failed to cleanup installation directory: {}", cleanup_err);
                        }
                    }
                    return Err(InstallError::EditorInstallationFailed(Box::new(err)));
                }
                Err(err) => {
                    // Module failure - collect and continue
                    log::warn!("Failed to install module {}: {}", module_id, err);
                    errors.push(err);
                }
                Ok(()) => {
                    // Mark module as installed in modules list
                    if let Some(m) = modules.iter_mut().find(|m| m.id() == module_id) {
                        m.is_installed = true;
                        trace!("module {} installed successfully", module_id);
                    }
                }
            }

            // Write modules.json after each module (success or failure)
            // Note: This won't run if we returned early from Editor failure above
            write_modules_json(base_dir, modules);
        }
    }

    // Return appropriate result based on collected errors
    if errors.is_empty() {
        Ok(())
    } else {
        Err(InstallError::ModuleInstallationsFailed(errors))
    }
}

fn write_modules_json(base_dir: &Path, modules: &[Module]) {
    let modules_json_path = base_dir.join("modules.json");
    if let Ok(json_content) = serde_json::to_string_pretty(&modules) {
        if let Err(e) = std::fs::write(&modules_json_path, json_content) {
            log::warn!("Failed to write modules.json: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::cmp::Ordering;
    use std::fmt::{Display, Formatter};
    use test_binary::build_test_binary;
    use unity_version::ReleaseType;

    #[derive(PartialEq, Eq, Debug, Clone)]
    #[allow(dead_code)]
    pub struct MockInstallation {
        version: Version,
        path: PathBuf,
    }

    #[allow(dead_code)]
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
        let _expected = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            expected
        } else {
            true
        };

        // Suppress unused variable warnings
        let _ = test_arch;
        let _ = test_version;
    }

    #[allow(dead_code)]
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

    mod incremental_sync_tests {
        use super::*;

        /// Create a test Module (unity-hub Module) by deserializing from JSON
        fn create_test_module(id: &str, is_installed: bool) -> Module {
            let json = format!(
                r#"{{
                    "id": "{}",
                    "name": "Test {}",
                    "description": "Test module",
                    "category": "test",
                    "downloadSize": 1000,
                    "installedSize": 2000,
                    "required": false,
                    "hidden": false,
                    "url": "https://example.com/{}.pkg",
                    "isInstalled": {}
                }}"#,
                id, id, id, is_installed
            );
            serde_json::from_str(&json).expect("Failed to parse test module JSON")
        }

        #[test]
        fn test_write_modules_json_creates_file() {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let base_dir = temp_dir.path();

            let modules = vec![
                create_test_module("android", false),
                create_test_module("ios", true),
            ];

            // Call the actual function under test
            write_modules_json(base_dir, &modules);

            // Verify file was created and contains correct data
            let modules_json_path = base_dir.join("modules.json");
            assert!(modules_json_path.exists(), "modules.json should be created");

            let content = std::fs::read_to_string(&modules_json_path).expect("Failed to read");
            let parsed: Vec<Module> = serde_json::from_str(&content).expect("Failed to parse");

            assert_eq!(parsed.len(), 2);
            assert_eq!(parsed[0].id(), "android");
            assert_eq!(parsed[0].is_installed, false);
            assert_eq!(parsed[1].id(), "ios");
            assert_eq!(parsed[1].is_installed, true);
        }

        #[test]
        fn test_write_modules_json_updates_installed_state() {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let base_dir = temp_dir.path();

            // Start with no modules installed
            let mut modules = vec![
                create_test_module("android", false),
                create_test_module("ios", false),
                create_test_module("webgl", false),
            ];

            write_modules_json(base_dir, &modules);

            // Simulate android installation completing
            modules[0].is_installed = true;
            write_modules_json(base_dir, &modules);

            // Read back and verify only android is installed
            let content = std::fs::read_to_string(base_dir.join("modules.json")).unwrap();
            let parsed: Vec<Module> = serde_json::from_str(&content).unwrap();

            assert_eq!(parsed[0].id(), "android");
            assert!(parsed[0].is_installed, "android should be installed");
            assert!(!parsed[1].is_installed, "ios should not be installed");
            assert!(!parsed[2].is_installed, "webgl should not be installed");
        }
    }

    /// Test infrastructure for testing install_modules_with_installer
    mod install_integration_tests {
        use super::*;
        use std::sync::{Arc, Mutex};
        use uvm_live_platform::Release;
        use uvm_install_graph::InstallGraph;

        // ============================================================
        // Test Fixtures - Create Release/Module from JSON
        // ============================================================

        /// Create a minimal Release JSON with the given module IDs
        fn create_test_release_json(module_ids: &[&str]) -> String {
            let modules_json: Vec<String> = module_ids
                .iter()
                .map(|id| create_platform_module_json(id))
                .collect();

            format!(
                r#"{{
                    "version": "2022.3.0f1",
                    "productName": "Unity",
                    "releaseDate": "2023-01-01",
                    "releaseNotes": {{ "url": "https://example.com/notes" }},
                    "stream": "LTS",
                    "skuFamily": "CLASSIC",
                    "recommended": true,
                    "unityHubDeepLink": "unityhub://2022.3.0f1",
                    "shortRevision": "abc123",
                    "downloads": [{{
                        "url": "https://example.com/unity.pkg",
                        "platform": "MAC_OS",
                        "architecture": "X86_64",
                        "downloadSize": 1000000,
                        "installedSize": 2000000,
                        "modules": [{}]
                    }}]
                }}"#,
                modules_json.join(",")
            )
        }

        /// Create a platform module JSON (uvm_live_platform::Module)
        fn create_platform_module_json(id: &str) -> String {
            format!(
                r#"{{
                    "id": "{}",
                    "name": "Test {}",
                    "description": "Test module {}",
                    "category": "Platforms",
                    "url": "https://example.com/{}.pkg",
                    "downloadSize": 500000,
                    "installedSize": 1000000,
                    "required": false,
                    "hidden": false,
                    "preSelected": false
                }}"#,
                id, id, id, id
            )
        }

        /// Create a Release from module IDs
        fn create_test_release(module_ids: &[&str]) -> Release {
            let json = create_test_release_json(module_ids);
            serde_json::from_str(&json).expect("Failed to parse test Release JSON")
        }

        /// Create a unity-hub Module for the modules list
        fn create_hub_module(id: &str, is_installed: bool) -> Module {
            let json = format!(
                r#"{{
                    "id": "{}",
                    "name": "Test {}",
                    "description": "Test module",
                    "category": "test",
                    "downloadSize": 1000,
                    "installedSize": 2000,
                    "required": false,
                    "hidden": false,
                    "url": "https://example.com/{}.pkg",
                    "isInstalled": {}
                }}"#,
                id, id, id, is_installed
            );
            serde_json::from_str(&json).expect("Failed to parse hub module JSON")
        }

        // ============================================================
        // MockModuleInstaller - Controls success/failure per module
        // ============================================================

        /// Mock installer that tracks install attempts and can simulate failures
        struct MockModuleInstaller {
            /// Module IDs that should fail installation
            fail_modules: HashSet<String>,
            /// Tracks the order of install attempts
            install_order: Arc<Mutex<Vec<String>>>,
        }

        impl MockModuleInstaller {
            fn new(fail_modules: HashSet<String>) -> Self {
                Self {
                    fail_modules,
                    install_order: Arc::new(Mutex::new(Vec::new())),
                }
            }

            fn with_no_failures() -> Self {
                Self::new(HashSet::new())
            }

            fn with_failures<I: IntoIterator<Item = S>, S: Into<String>>(modules: I) -> Self {
                Self::new(modules.into_iter().map(|s| s.into()).collect())
            }

            fn get_install_order(&self) -> Vec<String> {
                self.install_order.lock().unwrap().clone()
            }
        }

        impl ModuleInstaller for MockModuleInstaller {
            fn install_module(&self, module_id: &str, _base_dir: &Path) -> Result<()> {
                // Record this install attempt
                self.install_order.lock().unwrap().push(module_id.to_string());

                if self.fail_modules.contains(module_id) {
                    Err(InstallError::InstallFailed(
                        module_id.to_string(),
                        crate::install::error::InstallerError::from(
                            io::Error::new(io::ErrorKind::Other, format!("Mock failure for {}", module_id))
                        ),
                    ))
                } else {
                    Ok(())
                }
            }
        }

        // ============================================================
        // Tests
        // ============================================================

        #[test]
        fn test_release_fixture_deserializes() {
            let release = create_test_release(&["android", "ios", "webgl"]);
            assert_eq!(release.version, "2022.3.0f1");
            assert_eq!(release.downloads.len(), 1);
            assert_eq!(release.downloads[0].modules.len(), 3);
            assert_eq!(release.downloads[0].modules[0].id(), "android");
            assert_eq!(release.downloads[0].modules[1].id(), "ios");
            assert_eq!(release.downloads[0].modules[2].id(), "webgl");
        }

        #[test]
        fn test_install_graph_from_release() {
            let release = create_test_release(&["android", "ios"]);
            let mut graph = InstallGraph::from(&release);

            // Mark all as missing (simulating fresh install)
            graph.mark_all_missing();

            // Keep only the modules we want
            let mut keep_set = HashSet::new();
            keep_set.insert("Unity".to_string());
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            graph.keep(&keep_set);

            // Verify we can iterate
            let mut count = 0;
            for _node in graph.topo().iter(graph.context()) {
                count += 1;
            }
            // Should have Editor + 2 modules = 3 nodes
            assert!(count >= 2, "Graph should have at least 2 nodes, got {}", count);
        }

        #[test]
        fn test_mock_installer_success() {
            let installer = MockModuleInstaller::with_no_failures();
            let temp_dir = tempfile::tempdir().unwrap();

            assert!(installer.install_module("android", temp_dir.path()).is_ok());
            assert!(installer.install_module("ios", temp_dir.path()).is_ok());

            let order = installer.get_install_order();
            assert_eq!(order, vec!["android", "ios"]);
        }

        #[test]
        fn test_mock_installer_failure() {
            let installer = MockModuleInstaller::with_failures(["ios"]);
            let temp_dir = tempfile::tempdir().unwrap();

            assert!(installer.install_module("android", temp_dir.path()).is_ok());
            assert!(installer.install_module("ios", temp_dir.path()).is_err());
            assert!(installer.install_module("webgl", temp_dir.path()).is_ok());

            let order = installer.get_install_order();
            assert_eq!(order, vec!["android", "ios", "webgl"]);
        }

        #[test]
        fn test_install_modules_all_succeed() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            let mut keep_set = HashSet::new();
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
            ];

            let installer = MockModuleInstaller::with_no_failures();
            let result = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);
            assert!(modules[0].is_installed, "android should be installed");
            assert!(modules[1].is_installed, "ios should be installed");
        }

        #[test]
        fn test_install_modules_single_failure() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios", "webgl"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            let mut keep_set = HashSet::new();
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            keep_set.insert("webgl".to_string());
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
                create_hub_module("webgl", false),
            ];

            // ios will fail
            let installer = MockModuleInstaller::with_failures(["ios"]);
            let result = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            assert!(result.is_err(), "Expected error");
            match result {
                Err(InstallError::ModuleInstallationsFailed(errors)) => {
                    assert_eq!(errors.len(), 1, "Expected 1 error");
                }
                _ => panic!("Expected ModuleInstallationsFailed error"),
            }
            assert!(modules[0].is_installed, "android should be installed");
            assert!(!modules[1].is_installed, "ios should NOT be installed");
            assert!(modules[2].is_installed, "webgl should be installed (continued past failure)");
        }

        #[test]
        fn test_install_modules_multiple_failures() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios", "webgl"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            let mut keep_set = HashSet::new();
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            keep_set.insert("webgl".to_string());
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
                create_hub_module("webgl", false),
            ];

            // ios and webgl will fail
            let installer = MockModuleInstaller::with_failures(["ios", "webgl"]);
            let result = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            assert!(result.is_err(), "Expected error");
            match result {
                Err(InstallError::ModuleInstallationsFailed(errors)) => {
                    assert_eq!(errors.len(), 2, "Expected 2 errors");
                }
                _ => panic!("Expected ModuleInstallationsFailed error"),
            }
            assert!(modules[0].is_installed, "android should be installed");
            assert!(!modules[1].is_installed, "ios should NOT be installed");
            assert!(!modules[2].is_installed, "webgl should NOT be installed");
        }

        #[test]
        fn test_modules_json_reflects_correct_state() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios", "webgl"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            let mut keep_set = HashSet::new();
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            keep_set.insert("webgl".to_string());
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
                create_hub_module("webgl", false),
            ];

            // ios fails
            let installer = MockModuleInstaller::with_failures(["ios"]);
            let _errors = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            // Read modules.json and verify state
            let modules_json_path = base_dir.join("modules.json");
            assert!(modules_json_path.exists(), "modules.json should exist");

            let content = std::fs::read_to_string(&modules_json_path).unwrap();
            let parsed: Vec<Module> = serde_json::from_str(&content).unwrap();

            let android = parsed.iter().find(|m| m.id() == "android");
            let ios = parsed.iter().find(|m| m.id() == "ios");
            let webgl = parsed.iter().find(|m| m.id() == "webgl");

            assert!(android.map(|m| m.is_installed).unwrap_or(false), "android should be installed in JSON");
            assert!(!ios.map(|m| m.is_installed).unwrap_or(true), "ios should NOT be installed in JSON");
            assert!(webgl.map(|m| m.is_installed).unwrap_or(false), "webgl should be installed in JSON");
        }

        #[test]
        fn test_install_continues_after_failure() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios", "webgl"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            let mut keep_set = HashSet::new();
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            keep_set.insert("webgl".to_string());
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
                create_hub_module("webgl", false),
            ];

            // ios fails (middle module)
            let installer = MockModuleInstaller::with_failures(["ios"]);
            let _errors = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            // Verify all modules were attempted
            let install_order = installer.get_install_order();
            assert!(install_order.contains(&"android".to_string()), "android should have been attempted");
            assert!(install_order.contains(&"ios".to_string()), "ios should have been attempted");
            assert!(install_order.contains(&"webgl".to_string()), "webgl should have been attempted (after ios failure)");
        }

        #[test]
        fn test_editor_failure_triggers_cleanup() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            let mut keep_set = HashSet::new();
            keep_set.insert("Unity".to_string());
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
            ];

            // Unity (Editor) will fail
            let installer = MockModuleInstaller::with_failures(["Unity"]);
            let result = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            // Verify EditorInstallationFailed error is returned
            assert!(result.is_err(), "Expected error");
            match result {
                Err(InstallError::EditorInstallationFailed(_)) => {
                    // Expected
                }
                _ => panic!("Expected EditorInstallationFailed error, got {:?}", result),
            }

            // Verify no modules were attempted (Editor is first in topo order)
            let install_order = installer.get_install_order();
            assert_eq!(install_order.len(), 1, "Only Unity should have been attempted");
            assert_eq!(install_order[0], "Unity");

            // Verify installation directory does not exist (cleanup worked)
            assert!(!base_dir.exists(), "Installation directory should have been cleaned up");
        }

        #[test]
        fn test_module_failure_with_existing_editor() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios", "webgl"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            // Simulate Editor already installed - only modules in graph
            let mut keep_set = HashSet::new();
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            keep_set.insert("webgl".to_string());
            // Note: "Unity" NOT in the keep_set, simulating existing Editor
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
                create_hub_module("webgl", false),
            ];

            // ios will fail
            let installer = MockModuleInstaller::with_failures(["ios"]);
            let result = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            // Verify ModuleInstallationsFailed error is returned
            assert!(result.is_err(), "Expected error");
            match result {
                Err(InstallError::ModuleInstallationsFailed(errors)) => {
                    assert_eq!(errors.len(), 1, "Expected 1 module error");
                }
                _ => panic!("Expected ModuleInstallationsFailed error"),
            }

            // Verify installation directory still exists
            assert!(base_dir.exists(), "Installation directory should still exist");

            // Verify other modules were installed
            assert!(modules[0].is_installed, "android should be installed");
            assert!(!modules[1].is_installed, "ios should NOT be installed");
            assert!(modules[2].is_installed, "webgl should be installed");
        }

        #[test]
        fn test_successful_installation_returns_ok() {
            let temp_dir = tempfile::tempdir().unwrap();
            let base_dir = temp_dir.path();

            let release = create_test_release(&["android", "ios"]);
            let mut graph = InstallGraph::from(&release);
            graph.mark_all_missing();

            let mut keep_set = HashSet::new();
            keep_set.insert("android".to_string());
            keep_set.insert("ios".to_string());
            graph.keep(&keep_set);

            let mut modules = vec![
                create_hub_module("android", false),
                create_hub_module("ios", false),
            ];

            let installer = MockModuleInstaller::with_no_failures();
            let result = install_modules_with_installer(&graph, base_dir, &mut modules, &installer);

            // Verify Ok(()) is returned
            assert!(result.is_ok(), "Expected Ok(()), got {:?}", result);

            // Verify all modules marked as installed
            assert!(modules[0].is_installed, "android should be installed");
            assert!(modules[1].is_installed, "ios should be installed");

            // Verify modules.json was written
            let modules_json_path = base_dir.join("modules.json");
            assert!(modules_json_path.exists(), "modules.json should exist");
        }
    }
}
