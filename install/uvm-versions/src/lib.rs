#[macro_use]
extern crate serde_derive;




use serde;
use uvm_cli;
use uvm_core;
#[macro_use]
extern crate log;

use console::Style;
use console::Term;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};
use std::io;
use std::result;
use std::str::FromStr;
use uvm_cli::ColorOption;
use uvm_core::unity::Version;
use uvm_core::unity::VersionType;

#[derive(Debug, Deserialize)]
pub struct VersionsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_regex")]
    arg_pattern: Option<Regex>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_alpha: bool,
    flag_beta: bool,
    flag_final: bool,
    flag_patch: bool,
    flag_all: bool,
    flag_color: ColorOption,
}

fn deserialize_regex<'de, D>(deserializer: D) -> result::Result<Option<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Regex::from_str(&s)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
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

    pub fn pattern(&self) -> &Option<Regex> {
        &self.arg_pattern
    }
}

impl uvm_cli::Options for VersionsOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }

    fn debug(&self) -> bool {
        self.flag_debug
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

type MajorVersion = u64;
type VersionTypeMap = HashMap<VersionType, Version>;
type MajorVersionTypeMap = HashMap<MajorVersion, VersionTypeMap>;
type VersionSet = HashSet<Version>;
type MajorVersionMap = HashMap<MajorVersion, VersionSet>;

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    fn progress_draw_target<T>(options: &T) -> ProgressDrawTarget
    where
        T: uvm_cli::Options,
    {
        if options.debug() {
            ProgressDrawTarget::hidden()
        } else {
            ProgressDrawTarget::stderr()
        }
    }

    fn no_versions_message(&self, options: &VersionsOptions) -> String {
        let variants = options.list_variants();
        let base_message = format!("no versions found in `{:#}`", &variants.iter().format(", "));
        match options.pattern() {
            Some(p) => format!("{} matching the pattern {:?}", base_message, p),
            None => base_message,
        }
    }

    fn out_message(&self, options: &VersionsOptions) -> String {
        let variants = options.list_variants();
        let variants_format = match options.list_variants().len() {
            4 => "".to_string(),
            _ => format!("{:#} ", &variants.iter().format(", ")),
        };

        let base_message = if options.filter_versions() {
            format!("Latest {}versions to install", variants_format)
        } else {
            format!("All available {}versions to install", variants_format)
        };

        match options.pattern() {
            Some(p) => format!("{} matching the pattern `{:?}`:", base_message, p),
            None => format!("{}:", base_message),
        }
    }

    fn major_versions_map<I>(&self, versions: I) -> MajorVersionMap
    where
        I: Iterator<Item = Version>,
    {
        let version_type: MajorVersionMap = MajorVersionMap::with_capacity(10);
        versions.fold(version_type, |mut version_type_map, version| {
            use std::collections::hash_map::Entry::*;
            match version_type_map.entry(version.major()) {
                Occupied(mut entry) => {
                    entry.get_mut().insert(version);
                }
                Vacant(entry) => {
                    let versions: VersionSet = VersionSet::with_capacity(1);
                    entry.insert(versions);
                }
            };
            version_type_map
        })
    }

    fn major_release_type_map<I>(&self, versions: I) -> MajorVersionTypeMap
    where
        I: Iterator<Item = (MajorVersion, VersionSet)>,
    {
        let versions_filter: MajorVersionTypeMap = MajorVersionTypeMap::with_capacity(10);
        versions.fold(versions_filter, |mut filter, (major, versions)| {
            use std::collections::hash_map::Entry::*;
            let type_map: VersionTypeMap = VersionTypeMap::new();
            let type_map = versions.into_iter().fold(type_map, |mut map, mut version| {
                match map.entry(*version.release_type()) {
                    Occupied(mut entry) => {
                        let v = entry.get_mut();
                        if v < version.as_mut() {
                            *v = version;
                        }
                    }
                    Vacant(entry) => {
                        entry.insert(version);
                    }
                }
                map
            });
            filter.insert(major, type_map);
            filter
        })
    }

    fn filter_versions<I>(&self, versions: I) -> impl Iterator<Item = Version>
    where
        I: Iterator<Item = Version>,
    {
        let version_type = self.major_versions_map(versions).into_iter();
        let versions_filter = self.major_release_type_map(version_type).into_iter();
        versions_filter.flat_map(|(_major, types)| types.into_iter().map(|(_t, version)| version))
    }

    pub fn exec(&self, options: &VersionsOptions) -> io::Result<()> {
        let out_style = Style::new().cyan();
        let message_style = Style::new().green().bold();
        let warning_style = Style::new().yellow().bold();

        let variants = options.list_variants();
        let progress = ProgressBar::new_spinner();
        let spinner_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        progress.set_style(spinner_style);
        progress.set_draw_target(UvmCommand::progress_draw_target(options));
        progress.set_prefix(&format!(
            "search unity versions: {}",
            format!("{:#}", &variants.iter().format(", "))
        ));
        progress.enable_steady_tick(100);
        progress.tick();
        debug!("fetch versions list");
        let versions = uvm_core::unity::all_versions()?;
        let versions: Vec<Version> = if options.filter_versions() {
            self.filter_versions(versions)
                .filter_map(|version| match options.pattern() {
                    Some(p) if p.is_match(&version.to_string()) => Some(version),
                    Some(_) => None,
                    None => Some(version),
                }).collect()
        } else {
            versions
                .filter_map(|version| match options.pattern() {
                    Some(p) if p.is_match(&version.to_string()) => Some(version),
                    Some(_) => None,
                    None => Some(version),
                }).collect()
        };

        progress.finish_and_clear();

        if versions.is_empty() {
            self.stderr.write_line(&format!(
                "{}",
                warning_style.apply_to(self.no_versions_message(&options))
            ))?;
        } else {
            self.stderr.write_line(&format!(
                "{}",
                message_style.apply_to(self.out_message(&options))
            ))?;
        }

        for version in versions {
            if variants.contains(version.release_type()) {
                self.stdout
                    .write_line(&format!("{}", out_style.apply_to(version)))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_highest_versions() {
        let command = UvmCommand::new();
        let versions = vec![
            Version::f(2017, 1, 1, 0),
            Version::f(2017, 2, 1, 0),
            Version::f(2017, 3, 1, 0),
        ];

        let filtered: Vec<Version> = command.filter_versions(versions.into_iter()).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], Version::f(2017, 3, 1, 0));
    }

    #[test]
    fn filters_highest_major_versions() {
        let command = UvmCommand::new();
        let versions = vec![
            Version::f(2017, 1, 1, 0),
            Version::f(2017, 2, 1, 0),
            Version::f(2017, 3, 1, 0),
            Version::f(2018, 1, 1, 0),
            Version::f(2018, 2, 1, 0),
            Version::f(2018, 3, 1, 0),
        ];

        let filtered: Vec<Version> = command.filter_versions(versions.into_iter()).collect();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&Version::f(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::f(2018, 3, 1, 0)));
    }

    #[test]
    fn filters_highest_from_each_release_type() {
        let command = UvmCommand::new();
        let versions = vec![
            Version::f(2017, 1, 1, 0),
            Version::f(2017, 2, 1, 0),
            Version::f(2017, 3, 1, 0),
            Version::b(2017, 1, 1, 0),
            Version::b(2017, 2, 1, 0),
            Version::b(2017, 3, 1, 0),
            Version::p(2017, 1, 1, 0),
            Version::p(2017, 2, 1, 0),
            Version::p(2017, 3, 1, 0),
            Version::a(2017, 1, 1, 0),
            Version::a(2017, 2, 1, 0),
            Version::a(2017, 3, 1, 0),
        ];

        let filtered: Vec<Version> = command.filter_versions(versions.into_iter()).collect();
        assert_eq!(filtered.len(), 4);
        assert!(filtered.contains(&Version::a(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::b(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::p(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::f(2017, 3, 1, 0)));
    }

    #[test]
    fn filters_highest_from_major_and_each_release_type() {
        let command = UvmCommand::new();
        let versions = vec![
            Version::f(2017, 1, 1, 0),
            Version::f(2017, 2, 1, 0),
            Version::f(2017, 3, 1, 0),
            Version::b(2017, 1, 1, 0),
            Version::b(2017, 2, 1, 0),
            Version::b(2017, 3, 1, 0),
            Version::p(2017, 1, 1, 0),
            Version::p(2017, 2, 1, 0),
            Version::p(2017, 3, 1, 0),
            Version::a(2017, 1, 1, 0),
            Version::a(2017, 2, 1, 0),
            Version::a(2017, 3, 1, 0),
            Version::f(2018, 1, 1, 0),
            Version::f(2018, 2, 1, 0),
            Version::f(2018, 3, 1, 0),
            Version::b(2018, 1, 1, 0),
            Version::b(2018, 2, 1, 0),
            Version::b(2018, 3, 1, 0),
            Version::p(2018, 1, 1, 0),
            Version::p(2018, 2, 1, 0),
            Version::p(2018, 3, 1, 0),
            Version::a(2018, 1, 1, 0),
            Version::a(2018, 2, 1, 0),
            Version::a(2018, 3, 1, 0),
            Version::f(2019, 1, 1, 0),
            Version::f(2019, 2, 1, 0),
            Version::f(2019, 3, 1, 0),
            Version::b(2019, 1, 1, 0),
            Version::b(2019, 2, 1, 0),
            Version::b(2019, 3, 1, 0),
            Version::p(2019, 1, 1, 0),
            Version::p(2019, 2, 1, 0),
            Version::p(2019, 3, 1, 0),
            Version::a(2019, 1, 1, 0),
            Version::a(2019, 2, 1, 0),
            Version::a(2019, 3, 1, 0),
        ];

        let filtered: Vec<Version> = command.filter_versions(versions.into_iter()).collect();
        assert_eq!(filtered.len(), 12);
        assert!(filtered.contains(&Version::a(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::b(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::p(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::f(2017, 3, 1, 0)));

        assert!(filtered.contains(&Version::a(2018, 3, 1, 0)));
        assert!(filtered.contains(&Version::b(2018, 3, 1, 0)));
        assert!(filtered.contains(&Version::p(2018, 3, 1, 0)));
        assert!(filtered.contains(&Version::f(2018, 3, 1, 0)));

        assert!(filtered.contains(&Version::a(2019, 3, 1, 0)));
        assert!(filtered.contains(&Version::b(2019, 3, 1, 0)));
        assert!(filtered.contains(&Version::p(2019, 3, 1, 0)));
        assert!(filtered.contains(&Version::f(2019, 3, 1, 0)));
    }
}
