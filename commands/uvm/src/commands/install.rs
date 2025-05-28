use std::path::PathBuf;
use clap::Args;
use unity_version::Version;

#[derive(Args, Debug)]
pub struct InstallArgs {
    #[arg(short, long = "module")]
    pub modules: Option<Vec<String>>,

    #[arg(long = "with-sync")]
    pub sync: bool,

    pub editor_version: Version,

    pub destination: Option<PathBuf>,
}
