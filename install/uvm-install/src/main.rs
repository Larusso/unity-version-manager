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
  uvm-install [options] <version>
  uvm-install (-h | --help)

Options:
  -a, --all         list all versions or install all support packages
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
    #[serde(with = "unity_version_format")]
    arg_version: Version,
    flag_verbose: bool,
    flag_android: bool,
    flag_ios: bool,
    flag_webgl: bool,
    flag_mobile: bool,
    flag_linux: bool,
    flag_windows: bool,
    flag_desktop: bool,
    flag_color: ColorOption,
}

impl InstallOptions {

    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn install_variants(&self) -> Option<HashSet<InstallVariant>> {
        if self.flag_android || self.flag_ios || self.flag_webgl || self.flag_linux || self.flag_windows || self.flag_mobile || self.flag_desktop {
            let mut variants:HashSet<InstallVariant> = HashSet::with_capacity(5);
            if self.flag_mobile {
                variants.insert(InstallVariant::Android);
                variants.insert(InstallVariant::Ios);
                variants.insert(InstallVariant::WebGl);
            }

            if self.flag_desktop {
                variants.insert(InstallVariant::Linux);
                variants.insert(InstallVariant::Windows);
            }

            if self.flag_android {
                variants.insert(InstallVariant::Android);
            }

            if self.flag_ios {
                variants.insert(InstallVariant::Ios);
            }

            if self.flag_webgl {
                variants.insert(InstallVariant::WebGl);
            }

            if self.flag_windows {
                variants.insert(InstallVariant::Windows);
            }

            if self.flag_linux {
                variants.insert(InstallVariant::Linux);
            }
            return Some(variants)
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
    let options:InstallOptions = uvm_cli::get_options(USAGE).unwrap();

    println!("{:?}", options.install_variants());
}

mod unity_version_format {
    use uvm_core::unity::Version;
    use std::str::FromStr;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}
