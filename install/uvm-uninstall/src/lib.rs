#[macro_use]
extern crate serde_derive;





use console::style;
use console::Term;
use std::collections::HashSet;
use std::fs::remove_dir_all;
use std::io;
use std::io::Write;
use uvm_cli::ColorOption;
use uvm_cli::Options;
use uvm_core::error::*;
use uvm_core::install::InstallVariant;
use uvm_core::unity;
use uvm_core::unity::Component;
use uvm_core::unity::Version;

#[derive(Debug, Deserialize)]
pub struct UninstallOptions {
    arg_version: Version,
    flag_verbose: bool,
    flag_debug: bool,
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
        let mut variants: HashSet<InstallVariant> = HashSet::with_capacity(6);

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
        variants
    }
}

impl Options for UninstallOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn debug(&self) -> bool {
        self.flag_debug
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}

#[derive(Default)]
pub struct UvmCommand {}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {}
    }

    pub fn exec(&self, options: &UninstallOptions) -> Result<()> {
        let mut stderr = Term::stderr();
        let installation = unity::find_installation(&options.version())?;
        let installed: HashSet<Component> = installation.installed_components().collect();

        let to_uninstall: HashSet<Component> = options
            .install_variants()
            .into_iter()
            .map(|v| v.into())
            .collect();

        if to_uninstall.contains(&Component::Editor) {
            writeln!(
                stderr,
                "{}: {}",
                style("uninstall unity version").green(),
                options.version()
            ).ok();
            remove_dir_all(installation.path())?
        } else {
            if options.verbose() {
                writeln!(
                    stderr,
                    "{}: {}",
                    style("uninstall unity components").green(),
                    options.version()
                ).ok();
                writeln!(stderr, "{}", style("Components to uninstall:").green()).ok();
                for c in &to_uninstall {
                    writeln!(stderr, "{}", style(c).cyan()).ok();
                }

                let mut diff = to_uninstall.difference(&installed).peekable();
                if diff.peek().is_some() {
                    stderr.write_line("").ok();
                    writeln!(stderr, "{}", style("Skip variants not installed:").yellow()).ok();
                    for c in diff {
                        writeln!(stderr, "{}", style(c).yellow().bold()).ok();
                    }
                }
            }

            let mut diff = to_uninstall.intersection(&installed).peekable();
            if diff.peek().is_some() {
                stderr.write_line("Start Uninstall").ok();
                for c in diff {
                    if let Some(p) = c.installpath().map(|l| installation.path().join(l)) {
                        stderr.write_line(&format!("Remove {}", c)).ok();
                        remove_dir_all(p)?
                    }
                }
            } else {
                return Err(io::Error::new(io::ErrorKind::Other, "nothing to uninstall").into());
            }
        }
        Ok(())
    }
}
