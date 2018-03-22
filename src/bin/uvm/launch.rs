extern crate console;
extern crate uvm;

use std::process;
use console::style;
use std::env;
use std::io;
use std::path::{Path,PathBuf};

const USAGE: &'static str = "
uvm-current - Launch the current active version of unity.

Usage:
  uvm-launch [options] [<project-path>]
  uvm-launch (-h | --help)

Options:
  -v, --verbose                 print more output
  -p, --platform=<platform>     the build platform to open the project with
  --use-project-version         Will launch try to launch the project with the Unity version the project was created from.
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
        let version = uvm::dectect_project_version(&project_path)?;
        eprintln!("found project version {}", version.to_string());
        return uvm::find_installation(&version);
    }

    uvm::current_installation()
}

fn main() {
    let o = uvm::cli::get_launch_options(USAGE).unwrap();
    eprintln!("{:?}", o);

    let project_path = o.project_path.unwrap_or(env::current_dir().unwrap());

    let installtion = get_installation(&project_path, o.use_project_version).unwrap_or_else(|err| {
        eprintln!("{}", style(err).red());
        process::exit(1);
    });

    if o.verbose {
        eprintln!("launch unity version: {}", style(installtion.version().to_string()).cyan())
    }

    let mut command = process::Command::new(
        installtion
            .exec_path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    if let Some(platform) = o.platform {
        command.arg("-buildTarget").arg(platform.to_string());
    };

    command
        .arg("-projectPath")
        .arg(project_path.canonicalize().unwrap().to_str().unwrap());

    command.spawn().unwrap_or_else(|err| {
        let message = format!("Unable to launch unity at {}", installtion.path().display());
        eprintln!("{}", style(message).red());
        eprintln!("{}", err);
        process::exit(1);
    });
}
