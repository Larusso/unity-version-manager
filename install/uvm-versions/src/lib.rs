#[macro_use]
extern crate serde_derive;
extern crate uvm_cli;
extern crate uvm_core;
extern crate console;
extern crate uvm_install_core;
extern crate itertools;

use console::Style;
use console::Term;
use std::io;
use std::collections::HashSet;
use uvm_cli::ColorOption;
use uvm_cli::Options;
use uvm_core::unity::VersionType;
use itertools::Itertools;

#[derive(Debug, Deserialize)]
pub struct VersionsOptions {
    flag_verbose: bool,
    flag_beta: bool,
    flag_final: bool,
    flag_patch: bool,
    flag_all: bool,
    flag_color: ColorOption,
}

impl VersionsOptions {
    pub fn list_variants(&self) -> HashSet<VersionType> {
        let mut variants:HashSet<VersionType> = HashSet::with_capacity(3);

        if self.flag_beta || self.flag_all {
            variants.insert(VersionType::Beta);
        }

        if self.flag_patch || self.flag_all {
            variants.insert(VersionType::Patch);
        }

        if self.flag_final || self.flag_all || variants.is_empty() {
            variants.insert(VersionType::Final);
        }
        return variants;
    }
}

impl uvm_cli::Options for VersionsOptions {
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

    fn readCasksFromStdOut(&self, stdout:&Vec<u8>) -> String {
        return String::from_utf8_lossy(stdout).into_owned()
    }

    pub fn exec(&self, options:VersionsOptions) -> io::Result<()>
    {
        let out_style = Style::new().cyan();

        let variants = options.list_variants();
        for variant in options.list_variants() {
            uvm_install_core::ensure_tap_for_version_type(&variant).unwrap();
        }

        let output = uvm_core::brew::cask::search(&format!("/unity@.*?({}).*/", itertools::join(&variants, "|")))
            .and_then(std::process::Child::wait_with_output)
            .map(|out| self.readCasksFromStdOut(&out.stdout))?;

        self.stderr.write_line("Available Unity versions to install:")?;
        for cask in output.lines().filter(|line| line.starts_with("unity@")) {
            self.stdout.write_line(&format!("{}", out_style.apply_to(cask.split("@").last().unwrap())))?;
        }
        Ok(())
    }
}
