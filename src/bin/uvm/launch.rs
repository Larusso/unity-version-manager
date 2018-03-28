extern crate console;
extern crate uvm;

use std::process;
use console::style;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use uvm::cli::LaunchOptions;

const USAGE: &'static str = "
uvm-current - Launch the current active version of unity.

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
  -h, --help                    show this help message and exit
";

fn get_installation(
    project_path: &Path,
    use_project_version: bool,
) -> io::Result<uvm::Installation> {
    if use_project_version {
        let version = uvm::dectect_project_version(&project_path, None)?;
        return uvm::find_installation(&version);
    }

    uvm::current_installation()
}

fn launch(options: LaunchOptions) -> io::Result<()> {
    let project_path = uvm::detect_unity_project_dir(
        options.project_path().unwrap_or(&env::current_dir().unwrap()),
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
    let o:LaunchOptions = uvm::cli::get_options(USAGE).unwrap();

    launch(o).unwrap_or_else(|err| {
        let message = format!("Unable to launch unity");
        eprintln!("{}", style(message).red());
        eprintln!("{}", err);
        process::exit(1);
    });
}
