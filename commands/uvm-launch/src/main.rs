extern crate console;
extern crate uvm_cli;
extern crate uvm_core;

use console::style;
use std::env;
use std::path::Path;
use std::process;
use uvm_cli::{LaunchOptions, Options};
use uvm_core::Result;

const USAGE: &str = "
uvm-launch - Launch the current active version of unity.

Usage:
  uvm-launch [options] [<project-path>]
  uvm-launch (-h | --help)

Options:
  -v, --verbose                 print more output
  -r, --recursive               Detects a unity project recursivly from current working or <project-path> directory.
  -f, --force-project-version   Will launch try to launch the project with the Unity version the project was created from.
  -p, --platform=<platform>     the build platform to open the project with
                                possible values:
                                win32, win64, osx, linux, linux64, ios, android, web,
                                webstreamed, webgl, xboxone, ps4, psp2, wsaplayer, tizen, samsungtv
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help                    show this help message and exit
";

fn get_installation(
    project_path: &Path,
    use_project_version: bool,
) -> Result<uvm_core::Installation> {
    if use_project_version {
        let version = uvm_core::dectect_project_version(&project_path, None)?;
        return uvm_core::find_installation(&version);
    }

    uvm_core::current_installation()
}

fn launch(options: &LaunchOptions) -> Result<()> {
    let project_path = uvm_core::detect_unity_project_dir(
        options
            .project_path()
            .unwrap_or(&env::current_dir().unwrap()),
        options.recursive(),
    )?;

    if options.verbose() {
        eprintln!("launch project: {}", style(&project_path.display()).cyan())
    }

    let installtion = get_installation(&project_path, options.force_project_version())?;

    if options.verbose() {
        eprintln!(
            "launch unity version: {}",
            style(installtion.version().to_string()).cyan()
        )
    }

    let mut command = process::Command::new(
        installtion
            .exec_path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    if let Some(ref platform) = options.platform() {
        command.arg("-buildTarget").arg(platform.to_string());
    };

    command
        .arg("-projectPath")
        .arg(project_path.canonicalize().unwrap().to_str().unwrap());

    command.spawn()?;
    Ok(())
}

fn main() {
    let o: LaunchOptions = uvm_cli::get_options(USAGE).unwrap();

    launch(&o).unwrap_or_else(|err| {
        let message = "Unable to launch unity";
        eprintln!("{}", style(message).red());
        eprintln!("{}", style(err).red());
        process::exit(1);
    });
}
