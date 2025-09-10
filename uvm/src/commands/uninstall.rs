use clap::Args;
use unity_version::Version;

#[derive(Args, Debug)]
pub struct UninstallArgs {
    pub version: Version,
}
