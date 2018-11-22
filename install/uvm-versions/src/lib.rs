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
use std::collections::{HashMap, HashSet};
use std::io;
use uvm_cli::ColorOption;
use uvm_core::unity::Version;
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
        let mut variants: HashSet<VersionType> = HashSet::with_capacity(4);

        if self.has_variant_flags() {
            if self.flag_alpha {
                variants.insert(VersionType::Alpha);
            }

            if self.flag_beta {
                variants.insert(VersionType::Beta);
            }

            if self.flag_patch {
                variants.insert(VersionType::Patch);
            }

            if self.flag_final {
                variants.insert(VersionType::Final);
            }
        } else {
            variants.insert(VersionType::Alpha);
            variants.insert(VersionType::Beta);
            variants.insert(VersionType::Patch);
            variants.insert(VersionType::Final);
        }
        variants
    }

    fn has_variant_flags(&self) -> bool {
        self.flag_alpha || self.flag_beta || self.flag_patch || self.flag_final
    }

    pub fn filter_versions(&self) -> bool {
        !self.flag_all
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

impl Default for UvmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    fn out_message(&self, options: &VersionsOptions) -> String {
        let variants = options.list_variants();
        let variants_format = match options.list_variants().len() {
            4 => "".to_string(),
            _ => format!("{:#} ", &variants.iter().format(", ")),
        };

        if options.filter_versions() {
            format!("Latest {}versions to install:", variants_format)
        } else {
            format!("All available {}versions to install:", variants_format)
        }
    }

    pub fn exec(&self, options: &VersionsOptions) -> io::Result<()> {
        let out_style = Style::new().cyan();
        let message_style = Style::new().green().bold();

        let variants = options.list_variants();
        let progress = ProgressBar::new_spinner();
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        progress.set_style(spinner_style);
        progress.set_prefix(&format!(
            "search unity versions: {}",
            format!("{:#}", &variants.iter().format(", "))
        ));
        progress.enable_steady_tick(100);
        progress.tick();

        let versions = uvm_core::unity::all_versions()?;
        let versions: Vec<Version> = if options.filter_versions() {
            let version_type: HashMap<u64, HashSet<Version>> = HashMap::with_capacity(10);
            let version_type = versions.fold(version_type, |mut version_type_map, version| {
                let major = version.major();
                let mut versions: HashSet<Version> = HashSet::with_capacity(1);
                versions.insert(version);

                if let Some(old_versions) = version_type_map.insert(major, versions) {
                    let mut versions = version_type_map.get_mut(&major).unwrap();
                    for v in old_versions {
                        versions.insert(v);
                    }
                }

                version_type_map
            });

            let mut versions_filter: HashMap<u64, HashMap<VersionType, Version>> =
                HashMap::with_capacity(10);
            let versions_filter =
                version_type
                    .into_iter()
                    .fold(versions_filter, |mut filter, (major, versions)| {
                        let type_map: HashMap<VersionType, Version> = HashMap::new();
                        filter.insert(
                            major,
                            versions.into_iter().fold(type_map, |mut map, version| {
                                let t = *version.release_type();
                                let needs_update = match map.get(&t) {
                                    Some(v) if v > version.as_ref() => false,
                                    _ => true,
                                };

                                if needs_update {
                                    map.insert(t, version);
                                }
                                map
                            }),
                        );
                        filter
                    });

            versions_filter
                .into_iter()
                .flat_map(|(_major, types)| types.into_iter().map(|(_t, version)| version))
                .collect()
        } else {
            versions.collect()
        };

        progress.finish_and_clear();

        self.stderr.write_line(&format!(
            "{}",
            message_style.apply_to(self.out_message(&options))
        ))?;
        for version in versions {
            if variants.contains(version.release_type()) {
                self.stdout
                    .write_line(&format!("{}", out_style.apply_to(version)))?;
            }
        }
        Ok(())
    }
}
