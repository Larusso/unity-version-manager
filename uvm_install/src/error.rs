use thiserror::Error;
use unity_hub::unity::error::UnityError;
use uvm_live_platform::error::LivePlatformError;
use crate::install;

pub type Result<T> = std::result::Result<T, InstallError>;

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Failed to load Unity release: {0}")]
    ReleaseLoadFailure(#[from] LivePlatformError),

    #[error("Unable to lock install process: {0}")]
    LockProcessFailure(#[from] std::io::Error),

    #[error("Unable to load installation: {0}")]
    UnityError(#[from] UnityError),

    #[error("Module '{0}' not supported for version '{1}'")]
    UnsupportedModule(String, String),

    #[error("Loading installer failed: {0}")]
    LoadingInstallerFailed(#[source] install::error::InstallerError),

    #[error("failed to created installer: {0}")]
    InstallerCreatedFailed(#[source] install::error::InstallerError),

    #[error("Installation failed for module {0}: {1}")]
    InstallFailed(String, #[source] install::error::InstallerError),

    #[error("Hub error: {0}")]
    HubError(#[from]unity_hub::error::UnityHubError),

    #[error("{}", MultipleInstallFailures::format_errors(.0))]
    MultipleInstallFailures(Vec<InstallError>),
}

/// Helper struct for formatting multiple errors
pub struct MultipleInstallFailures;

impl MultipleInstallFailures {
    fn format_errors(errors: &[InstallError]) -> String {
        if errors.is_empty() {
            return "No errors".to_string();
        }
        if errors.len() == 1 {
            return format!("1 module failed to install: {}", errors[0]);
        }
        let mut msg = format!("{} modules failed to install:\n", errors.len());
        for (i, err) in errors.iter().enumerate() {
            msg.push_str(&format!("  {}. {}\n", i + 1, err));
        }
        msg
    }
}

// impl_context!(InstallError(InstallError));