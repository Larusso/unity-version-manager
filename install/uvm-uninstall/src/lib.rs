#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate serde;
extern crate uvm_cli;
extern crate uvm_core;

use console::style;
use console::Term;
use std::collections::HashSet;
use std::io;
use std::io::Write;
use uvm_cli::ColorOption;
use uvm_cli::Options;
use uvm_core::brew;
use uvm_core::unity::Version;
use uvm_core::install;
use uvm_core::install::InstallVariant;

#[derive(Debug, Deserialize)]
pub struct UninstallOptions {
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

    pub fn install_variants(&self) -> HashSet<InstallVariant> {
        let mut variants:HashSet<InstallVariant> = HashSet::with_capacity(6);

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

        if self.flag_all || variants.is_empty() {
            variants.insert(InstallVariant::Editor);
            variants.insert(InstallVariant::Android);
            variants.insert(InstallVariant::Ios);
            variants.insert(InstallVariant::WebGl);
            variants.insert(InstallVariant::Windows);
            variants.insert(InstallVariant::Linux);
        }
        return variants
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

pub struct UvmCommand {
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {}
    }

    pub fn exec(&self, options:UninstallOptions) -> io::Result<()> {
        let mut stderr = Term::stderr();
        write!(stderr, "{}: {}\n", style("uninstall unity version").green(), options.version().to_string()).ok();

        let casks = brew::cask::list()?;
        let installed: HashSet<brew::cask::Cask> = casks
            .filter(|cask| cask.contains(&format!("@{}", &options.version().to_string())))
            .collect();

        let mut to_uninstall = HashSet::new();

        for variant in options.install_variants() {
            to_uninstall.insert(install::cask_name_for_type_version(variant, &options.version()));
        }

        if options.verbose() {
            write!(stderr, "{}\n", style("Casks to uninstall:").green()).ok();
            for c in &to_uninstall {
                write!(stderr, "{}\n", style(c).cyan()).ok();
            }

            let mut diff = to_uninstall.difference(&installed).peekable();
            if let Some(_) = diff.peek() {
                stderr.write_line("").ok();
                write!(stderr, "{}\n", style("Skip variants not installed:").yellow()).ok();
                for c in diff {
                    write!(stderr, "{}\n", style(c).yellow().bold()).ok();
                }
            }
        }

        let mut diff = to_uninstall.intersection(&installed).peekable();
        if let Some(_) = diff.peek() {
            stderr.write_line("Start Uninstall").ok();
            let mut child = brew::cask::uninstall(diff)?;
            let status = child.wait()?;

            if !status.success() {
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to uninstall casks"));
            }
        }
        else {
            return Err(io::Error::new(io::ErrorKind::Other, "Version and all support packages not installed"));
        }

        Ok(())
    }
}
