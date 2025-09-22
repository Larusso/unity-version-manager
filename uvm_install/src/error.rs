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
}

// impl_context!(InstallError(InstallError));