use thiserror::Error;
use uvm_live_platform::error::LivePlatformError;

pub type Result<T> = std::result::Result<T, InstallError>;

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Failed to load Unity release")]
    ReleaseLoadFailure(#[from] LivePlatformError),

    #[error("Unable to lock install process")]
    LockProcessFailure(#[from] std::io::Error),
}