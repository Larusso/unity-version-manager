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
}

fn main() {
    install(uvm_cli::get_options(USAGE).unwrap()).unwrap_or_else(|err| {
        let message = format!("Unable to install Unity");
        eprintln!("{}", style(message).red());
        eprintln!("{}", style(err).red());
        process::exit(1);
    });
}

fn install(options: InstallOptions) -> io::Result<()> {
    ensure_tap_for_version(&options.version())?;
    // def install version: :latest, **support_package_options
    //   ensure_tap_for_version version
    //
    //   installed = cask.list.select {|cask| cask.include? "@#{version}"}
    //
    //   to_install = []
    //   to_install << cask_name_for_type_version(:unity, version)
    //   to_install += check_support_packages version, **support_package_options
    //   to_install = to_install - installed
    //
    //   cask.install(*to_install) unless to_install.empty?
    // end
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
    use std::io;

    pub mod cask {
        use std::io;

        pub fn list() -> io::Result<()> {
            Ok(())
        }

        pub fn install(cask: &str) -> io::Result<()> {
            Ok(())
        }
    }

    pub mod tap {
        use std::io;
        use std::process::Command;

        pub fn contains(tap_name: &str) -> bool {
            if let Ok(output) = Command::new("brew").arg("tap").output() {
                let str_err = String::from_utf8_lossy(&output.stderr);
                return str_err.contains(tap_name)
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
                return add(tap_name)
            }
            Ok(())
        }
    }

}
