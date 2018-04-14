#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate serde;
extern crate uvm_cli;
extern crate uvm_core;

use uvm_cli::Options;
use uvm_cli::ColorOption;
use std::collections::HashSet;
use uvm_core::unity::Version;
use uvm_core::unity::VersionType;

use console::style;
use std::process;
use std::io;
use std::fmt;

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
            || self.flag_windows || self.flag_mobile || self.flag_desktop
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
    install(uvm_cli::get_options(USAGE).unwrap()).unwrap_or_else(|err| {
        let message = format!("Unable to install Unity");
        eprintln!("{}", style(message).red());
        eprintln!("{}", style(err).red());
        process::exit(1);
    });

    eprintln!("Finish");
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
    ensure_tap_for_version(&options.version())?;
    let casks = brew::cask::list()?;
    let taps = brew::tap::list()?;

    for tap in taps {
        println!("{}", tap);
    }

    let installed: HashSet<brew::cask::Cask> = casks
        .into_iter()
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
        eprintln!("Choosen casks to install:");
        for c in &to_install {
            eprintln!("{}", c);
        }
    }

    let args = to_install
        .difference(&installed)
        .fold(String::new(), |acc, ref cask| acc + " " + cask);
    if options.verbose() {
        println!("Install with args {}", args);
    }

    let mut child = brew::cask::install(&args)?;
    let status = child.wait()?;
    if status.success() {
        println!("'projects/' directory created");
    } else {
        println!("failed to create 'projects/' directory");
    }
    Ok(())
}

fn ensure_tap_for_version(version: &Version) -> io::Result<()> {
    match *version.release_type() {
        VersionType::Final => brew::tap::ensure("wooga/unityversions"),
        VersionType::Beta => brew::tap::ensure("wooga/unityversions-beta"),
        VersionType::Patch => brew::tap::ensure("wooga/unityversions-patch"),
    }
}

mod brew {
    pub mod cask {
        use std::io;
        use std::process::Command;
        use std::process::Child;
        use std::str;

        pub type Cask = String;
        pub type Casks = Vec<Cask>;

        pub fn list() -> io::Result<Casks> {
            Command::new("brew")
                .arg("tap")
                .output()
                .map(|o| o.stdout)
                .and_then(|stdout| {
                    let out = str::from_utf8(&stdout)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    Ok(out.lines().map(|line| Cask::from(line)).collect())
                })
        }

        pub fn install(cask: &str) -> io::Result<Child> {
            Command::new("brew")
                .arg("cask")
                .arg("install")
                .arg(cask.trim())
                .spawn()
        }
    }

    pub mod tap {
        use std::io;
        use std::process::Command;
        use std::path::Path;
        use std::fs;

        const BREW_TAPS_LOCATION: &'static str = "/usr/local/Homebrew/Library/Taps";

        pub struct Taps(Box<Iterator<Item = String>>);

        impl Taps {
            fn new() -> io::Result<Taps> {
                let read_dir = fs::read_dir(BREW_TAPS_LOCATION)?;
                let iter = read_dir
                    .filter_map(io::Result::ok)
                    .flat_map(|d| {
                        let inner_read = fs::read_dir(d.path()).expect("read dir");
                        inner_read.filter_map(io::Result::ok)
                    })
                    .map(|d| {
                        let path = d.path();
                        let parent = path.parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap();
                        let tap_name = path.file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace("homebrew-","");
                        format!("{}/{}", parent, tap_name)
                    });
                Ok(Taps(Box::new(iter)))
            }
        }

        impl Iterator for Taps {
            type Item = String;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }

        pub fn list() -> io::Result<Taps> {
            Taps::new()
        }

        pub fn contains(tap_name: &str) -> bool {
            if let Ok(l) = list() {
                return l.collect::<Vec<String>>().contains(&String::from(tap_name))
            }
            false
        }

        pub fn add(tap_name: &str) -> io::Result<()> {
            let output = Command::new("brew").args(&["tap", tap_name]).output()?;
            if output.status.success() {
                return Ok(());
            }
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "failed to add tap:/n{}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }

        pub fn ensure(tap_name: &str) -> io::Result<()> {
            if !contains(tap_name) {
                return add(tap_name);
            }
            Ok(())
        }
    }

}
