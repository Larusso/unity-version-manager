use anyhow::{Context, Result};
use console::style;
use log::*;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use structopt::{
    clap::arg_enum, clap::crate_authors, clap::crate_description, clap::crate_version, StructOpt,
};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// the build platform to open the project with
    #[structopt(short, long, possible_values = &UnityPlatform::variants(), case_insensitive = true)]
    platform: Option<UnityPlatform>,

    /// print more output
    #[structopt(short, long, parse(from_occurrences))]
    verbose: i32,

    /// Detects a unity project recursivly from current working or <project-path> directory.
    #[structopt(short, long)]
    recursive: bool,

    /// Will launch try to launch the project with the Unity version the project was created from.
    #[structopt(short, long)]
    force_project_version: bool,

    /// Color:.
    #[structopt(short, long, possible_values = &ColorOption::variants(), case_insensitive = true, default_value)]
    color: ColorOption,

    /// Path to the Unity Project
    #[structopt(parse(from_os_str))]
    project_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opts::from_args();
    set_colors_enabled(&opt.color);
    set_loglevel(opt.verbose);
    
    launch(&opt).context("failed to launch Unity")?;
    Ok(())
}

arg_enum! {
#[derive(Debug, Clone)]
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
}

fn get_installation(
    project_path: &Path,
    use_project_version: bool,
) -> uvm_core::error::Result<uvm_core::Installation> {
    if use_project_version {
        let version = uvm_core::dectect_project_version(&project_path, None)?;
        let installation = uvm_core::find_installation(&version)?;
        return Ok(installation);
    }

    let installation = uvm_core::current_installation()?;
    Ok(installation)
}

fn launch(options: &Opts) -> Result<()> {
    let project_path = options
        .project_path
        .as_ref()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| env::current_dir().expect("current working directory"));
    let project_path = uvm_core::detect_unity_project_dir(&project_path, options.recursive)?;

    info!("launch project: {}", style(&project_path.display()).cyan());

    let installtion = get_installation(&project_path, options.force_project_version)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to fetch unity installation!"))?;

    info!(
        "launch unity version: {}",
        style(installtion.version().to_string()).cyan()
    );

    let mut command = process::Command::new(
        installtion
            .exec_path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    if let Some(ref platform) = options.platform {
        command.arg("-buildTarget").arg(platform.to_string());
    };

    command
        .arg("-projectPath")
        .arg(project_path.canonicalize().unwrap().to_str().unwrap());

    command.spawn()?;
    Ok(())
}
