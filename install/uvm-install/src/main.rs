#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate serde;
extern crate uvm_cli;
extern crate uvm_core;

use console::Style;
use std::io::Write;
use console::Term;
use uvm_cli::Options;
use uvm_cli::ColorOption;
use std::collections::HashSet;
use uvm_core::unity::Version;
use uvm_core::unity::VersionType;

use console::style;
use std::process;
use std::io;
use std::fmt;
use uvm_core::brew;

const USAGE: &'static str = "
uvm-install - Install specified unity version.

Usage:
  uvm-install [options] <version>
  uvm-install (-h | --help)

Options:
  -a, --all         install all support packages
  --android         install android support for editor
  --ios             install ios support for editor
  --webgl           install webgl support for editor
  --mobile          install mobile support (android, ios, webgl)
  --linux           install linux support for editor
  --windows         install windows support for editor
  --desktop         install desktop support (linux, windows)
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

#[derive(Debug, Deserialize)]
struct InstallOptions {
    #[serde(with = "uvm_core::unity::unity_version_format")]
    arg_version: Version,
    flag_verbose: bool,
    flag_android: bool,
    flag_ios: bool,
    flag_webgl: bool,
    flag_mobile: bool,
    flag_linux: bool,
    flag_windows: bool,
    flag_desktop: bool,
    flag_all: bool,
    flag_color: ColorOption,
}

impl InstallOptions {
    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn install_variants(&self) -> Option<HashSet<InstallVariant>> {
        if self.flag_android || self.flag_ios || self.flag_webgl || self.flag_linux
            || self.flag_windows || self.flag_mobile || self.flag_desktop || self.flag_all
        {
            let mut variants: HashSet<InstallVariant> = HashSet::with_capacity(5);

            if self.flag_android || self.flag_mobile || self.flag_all {
                variants.insert(InstallVariant::Android);
            }

            if self.flag_ios || self.flag_mobile || self.flag_all {
                variants.insert(InstallVariant::Ios);
            }

            if self.flag_webgl || self.flag_mobile || self.flag_all {
                variants.insert(InstallVariant::WebGl);
            }

            if self.flag_windows || self.flag_desktop || self.flag_all {
                variants.insert(InstallVariant::Windows);
            }

            if self.flag_linux || self.flag_desktop || self.flag_all {
                variants.insert(InstallVariant::Linux);
            }
            return Some(variants);
        }
        None
    }
}

impl Options for InstallOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
enum InstallVariant {
    Android,
    Ios,
    WebGl,
    Linux,
    Windows,
    Editor,
}

impl fmt::Display for InstallVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &InstallVariant::Android => write!(f, "android"),
            &InstallVariant::Ios => write!(f, "ios"),
            &InstallVariant::WebGl => write!(f, "webgl"),
            &InstallVariant::Linux => write!(f, "linux"),
            &InstallVariant::Windows => write!(f, "windows"),
            _ => write!(f, "editor"),
        }
    }
}

fn main() {
    let mut stdout = Term::stderr();
    install(uvm_cli::get_options(USAGE).unwrap()).unwrap_or_else(|err| {
        let message = format!("Unable to install");
        write!(stdout, "{}\n", style(message).red()).ok();
        write!(stdout, "{}\n", style(err).red()).ok();
        process::exit(1);
    });

    stdout.write_line("Finish").ok();
}

fn cask_name_for_type_version(variant: InstallVariant, version: &Version) -> brew::cask::Cask {
    let base_name = if variant == InstallVariant::Editor {
        String::from("unity")
    } else {
        format!("unity-{}-support-for-editor", variant)
    };

    String::from(format!("{}@{}", base_name, version.to_string()))
}

fn install(options: InstallOptions) -> io::Result<()> {
    let mut stderr = Term::stderr();
    write!(stderr, "{}: {}\n", style("install unity version").green(), options.version().to_string()).ok();


    ensure_tap_for_version(&options.version())?;

    let casks = brew::cask::list()?;
    let installed: HashSet<brew::cask::Cask> = casks
        .filter(|cask| cask.contains(&format!("@{}", &options.version().to_string())))
        .collect();

    let mut to_install = HashSet::new();
    to_install.insert(cask_name_for_type_version(
        InstallVariant::Editor,
        &options.version(),
    ));

    if let Some(variants) = options.install_variants() {
        for variant in variants {
            to_install.insert(cask_name_for_type_version(variant, &options.version()));
        }
    }

    if options.verbose() {
        write!(stderr, "{}\n", style("Casks to install:").green()).ok();
        for c in &to_install {
            write!(stderr, "{}\n", style(c).cyan()).ok();
        }

        let mut diff = to_install.union(&installed).peekable();
        if let Some(_) = diff.peek() {
            stderr.write_line("").ok();
            write!(stderr, "{}\n", style("Skip variants already installed:").yellow()).ok();
            for c in diff {
                write!(stderr, "{}\n", style(c).yellow().bold()).ok();
            }
        }
    }

    let mut diff = to_install.difference(&installed).peekable();
    if let Some(_) = diff.peek() {
        let mut child = brew::cask::install(diff)?;
        let status = child.wait()?;

        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to install casks"));
        }
    }
    else {
        return Err(io::Error::new(io::ErrorKind::Other, "Version and all support packages already installed"));
    }

    Ok(())
}

fn ensure_tap_for_version(version: &Version) -> io::Result<()> {
    let tap = match *version.release_type() {
        VersionType::Final => "wooga/unityversions",
        VersionType::Beta => "wooga/unityversions-beta",
        VersionType::Patch => "wooga/unityversions-patch",
    };
    brew::tap::ensure(tap)
}
