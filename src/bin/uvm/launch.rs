extern crate console;
extern crate uvm;

use std::process;
use console::style;

const USAGE: &'static str = "
uvm-current - Launch the current active version of unity.

Usage:
  uvm-launch [options] [<project-path>]
  uvm-launch (-h | --help)

Options:
  -v, --verbose                 print more output
  -p, --platform=<platform>     the build platform to open the project with
                                possible values:
                                win32, win64, osx, linux, linux64, ios, android, web,
                                webstreamed, webgl, xboxone, ps4, psp2, wsaplayer, tizen, samsungtv
  -h, --help                    show this help message and exit
";

fn main() {
    let o = uvm::cli::get_launch_options(USAGE).unwrap();
    if let Ok(installtion) = uvm::current_installation() {
        let mut command = process::Command::new(
            installtion.exec_path().canonicalize().unwrap().to_str().unwrap()
        );

        if let Some(platform) = o.platform {
            command.arg("-buildTarget").arg(platform.to_string());
        };

        if let Some(project_path) = o.project_path {
            command.arg("-projectPath").arg(project_path.canonicalize().unwrap().to_str().unwrap());
        }

        command.spawn().unwrap_or_else(|err| {
            let message = format!("Unable to launch unity at {}", installtion.path().display());
            eprintln!("{}", style(message).red());
            eprintln!("{}", err);
            process::exit(1);
        });
    }
}
