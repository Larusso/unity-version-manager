#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate indicatif;
extern crate itertools;
extern crate uvm_cli;
extern crate uvm_core;

use console::Style;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::collections::HashSet;
use std::io;
use uvm_cli::ColorOption;
use uvm_core::install;
use uvm_core::unity::VersionType;

#[derive(Debug, Deserialize)]
pub struct VersionsOptions {
    flag_verbose: bool,
    flag_alpha: bool,
    flag_beta: bool,
    flag_final: bool,
    flag_patch: bool,
    flag_all: bool,
    flag_color: ColorOption,
}

impl VersionsOptions {
    pub fn list_variants(&self) -> HashSet<VersionType> {
        let mut variants: HashSet<VersionType> = HashSet::with_capacity(3);

        if self.flag_alpha || self.flag_all {
            variants.insert(VersionType::Alpha);
        }

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
    stderr: Term,
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    fn read_casks_from_std_out(&self, stdout: &Vec<u8>) -> String {
        return String::from_utf8_lossy(stdout).into_owned();
    }

    pub fn exec(&self, options: VersionsOptions) -> io::Result<()> {
        let out_style = Style::new().cyan();

        let variants = options.list_variants();
        for variant in options.list_variants() {
            install::ensure_tap_for_version_type(&variant).unwrap();
        }

        let bar = ProgressBar::new_spinner();
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        bar.set_style(spinner_style);
        bar.set_prefix(&format!(
            "search unity versions: {}",
            format!("{:#}", &variants.iter().format(", "))
        ));
        bar.enable_steady_tick(100);
        bar.tick();

        let output =
            uvm_core::brew::cask::search(&format!("/unity@.*?({}).*/", &variants.iter().join("|")))
                .and_then(std::process::Child::wait_with_output)
                .map(|out| self.read_casks_from_std_out(&out.stdout))?;

        bar.finish_and_clear();

        self.stderr
            .write_line("Available Unity versions to install:")?;
        for cask in output.lines().filter(|line| line.starts_with("unity@")) {
            self.stdout.write_line(&format!(
                "{}",
                out_style.apply_to(cask.split("@").last().unwrap())
            ))?;
        }
        Ok(())
    }
}
