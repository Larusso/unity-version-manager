#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate uvm_cli;
extern crate uvm_core;
extern crate console;

use uvm_cli::Options;
use uvm_cli::ColorOption;
use std::collections::HashSet;
use uvm_core::unity::Version;

const USAGE: &'static str = "
uvm-install - Install specified unity version.

Usage:
  uvm-uninstall [options] <version>
  uvm-uninstall (-h | --help)

Options:
  -a, --all         uninstall all support packages
  --android         uninstall android support for editor
  --ios             uninstall ios support for editor
  --webgl           uninstall webgl support for editor
  --mobile          uninstall mobile support (android, ios, webgl)
  --linux           uninstall linux support for editor
  --windows         uninstall windows support for editor
  --desktop         uninstall desktop support (linux, windows)
  -v, --verbose     print more output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

#[derive(Debug, Deserialize)]
struct UninstallOptions {
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

impl UninstallOptions {

    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn install_variants(&self) -> Option<HashSet<InstallVariant>> {
        if self.flag_android || self.flag_ios || self.flag_webgl || self.flag_linux || self.flag_windows || self.flag_mobile || self.flag_desktop {
            let mut variants:HashSet<InstallVariant> = HashSet::with_capacity(5);

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
            return Some(variants)
        }
        None
    }
}

impl Options for UninstallOptions {
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
    let options:UninstallOptions = uvm_cli::get_options(USAGE).unwrap();

    println!("{:?}", options.install_variants());
}
