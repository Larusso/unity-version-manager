#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate serde;
extern crate uvm_cli;
extern crate uvm_core;

use std::io::Write;
use console::Term;
use uvm_cli::ColorOption;
use std::collections::HashSet;
use uvm_core::unity::Version;
use uvm_core::install;
use uvm_core::install::InstallVariant;
use std::str::FromStr;

use console::style;
use std::process;
use std::io;
use uvm_core::brew;

#[derive(Debug, Deserialize)]
pub struct Options {
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

impl Options {
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

            let check_version = Version::from_str("2018.0.0b0").unwrap();
            if (self.flag_windows || self.flag_desktop || self.flag_all) && self.version() >= &check_version {
                variants.insert(InstallVariant::WindowsMono);
            }

            if (self.flag_windows || self.flag_desktop || self.flag_all) && self.version() < &check_version {
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

impl uvm_cli::Options for Options {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}

pub struct UvmCommand {
    stdout: Term,
    stderr: Term
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    pub fn exec(&self, options:Options) -> io::Result<()> {
        write!(Term::stderr(), "{}: {}\n", style("install unity version").green(), options.version().to_string()).ok();

        install::ensure_tap_for_version(&options.version())?;

        let casks = brew::cask::list()?;
        let installed: HashSet<brew::cask::Cask> = casks
            .filter(|cask| cask.contains(&format!("@{}", &options.version().to_string())))
            .collect();

        let mut to_install = HashSet::new();
        to_install.insert(install::cask_name_for_type_version(
            InstallVariant::Editor,
            &options.version(),
        ));

        if let Some(variants) = options.install_variants() {
            for variant in variants {
                to_install.insert(install::cask_name_for_type_version(variant, &options.version()));
            }
        }

        if options.flag_verbose {
            write!(Term::stderr(), "{}\n", style("Casks to install:").green()).ok();
            for c in &to_install {
                write!(Term::stderr(), "{}\n", style(c).cyan()).ok();
            }

            let mut diff = to_install.union(&installed).peekable();
            if let Some(_) = diff.peek() {
                self.stderr.write_line("").ok();
                write!(Term::stderr(), "{}\n", style("Skip variants already installed:").yellow()).ok();
                for c in diff {
                    write!(Term::stderr(), "{}\n", style(c).yellow().bold()).ok();
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
}
