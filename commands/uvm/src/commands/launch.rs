use std::path::PathBuf;
use clap::{Args, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum UnityPlatform {
    Win32,
    Win64,
    OSX,
    Linux,
    Linux64,
    IOS,
    Android,
    Web,
    WebStreamed,
    WebGl,
    XboxOne,
    PS4,
    PSP2,
    WsaPlayer,
    Tizen,
    SamsungTV,
}

#[derive(Args, Debug)]
pub struct LaunchArgs {
    #[arg(short, long)]
    pub platform: Option<UnityPlatform>,

    #[arg(short, long)]
    pub recursive: bool,

    #[arg(short, long)]
    pub force_project_version: bool,

    #[arg()]
    pub project_path: Option<PathBuf>,
}