use console::Style;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use itertools::Itertools;
use log::debug;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io;
use uvm_core;
use uvm_core::unity::Version;
use uvm_core::unity::VersionType;

pub trait VersionListOptions {
    fn list_variants(&self) -> HashSet<VersionType>;
    fn has_variant_flags(&self) -> bool;
    fn filter_versions(&self) -> bool;
    fn pattern(&self) -> &Option<Regex>;
    fn debug(&self) -> bool;
}

type MajorVersion = u64;
type VersionTypeMap = HashMap<VersionType, Version>;
type MajorVersionTypeMap = HashMap<MajorVersion, VersionTypeMap>;
type VersionSet = HashSet<Version>;
type MajorVersionMap = HashMap<MajorVersion, VersionSet>;

pub fn list_versions<O: VersionListOptions>(options: &O) -> io::Result<()> {
    let out_style = Style::new().cyan();
    let message_style = Style::new().green().bold();
    let warning_style = Style::new().yellow().bold();

    let variants = options.list_variants();
    let progress = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    progress.set_style(spinner_style);
    progress.set_draw_target(progress_draw_target(options));
    progress.set_prefix(&format!(
        "search api versions: {}",
        format!("{:#}", &variants.iter().format(", "))
    ));
    progress.enable_steady_tick(100);
    progress.tick();
    debug!("fetch versions list");
    let versions = uvm_core::unity::all_versions().map_err(|err| std::io::Error::new(std::io::ErrorKind::NotFound, err))?;
    let versions: Vec<Version> = if options.filter_versions() {
        filter_versions(versions)
            .filter(|version| match options.pattern() {
                Some(p) if p.is_match(&version.to_string()) => true,
                Some(_) => false,
                None => true,
            })
            .collect()
    } else {
        versions
            .filter(|version| match options.pattern() {
                Some(p) if p.is_match(&version.to_string()) => true,
                Some(_) => false,
                None => true,
            })
            .collect()
    };

    progress.finish_and_clear();

    if versions.is_empty() {
        eprintln!("{}", warning_style.apply_to(no_versions_message(options)))
    } else {
        eprintln!("{}", message_style.apply_to(out_message(options)))
    }

    for version in versions {
        if variants.contains(version.release_type()) {
            println!("{}", out_style.apply_to(version));
        }
    }
    Ok(())
}

fn progress_draw_target<T>(options: &T) -> ProgressDrawTarget
where
    T: VersionListOptions,
{
    if options.debug() {
        ProgressDrawTarget::hidden()
    } else {
        ProgressDrawTarget::stderr()
    }
}

fn no_versions_message<O: VersionListOptions>(options: &O) -> String {
    let variants = options.list_variants();
    let base_message = format!("no versions found in `{:#}`", &variants.iter().format(", "));
    match options.pattern() {
        Some(p) => format!("{} matching the pattern {:?}", base_message, p),
        None => base_message,
    }
}

fn out_message<O: VersionListOptions>(options: &O) -> String {
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

fn major_versions_map<I>(versions: I) -> MajorVersionMap
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

fn major_release_type_map<I>(versions: I) -> MajorVersionTypeMap
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

fn filter_versions<I>(versions: I) -> impl Iterator<Item = Version>
where
    I: Iterator<Item = Version>,
{
    let version_type = major_versions_map(versions).into_iter();
    let versions_filter = major_release_type_map(version_type).into_iter();
    versions_filter.flat_map(|(_major, types)| types.into_iter().map(|(_t, version)| version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_highest_versions() {
        let versions = vec![
            Version::f(2017, 1, 1, 0),
            Version::f(2017, 2, 1, 0),
            Version::f(2017, 3, 1, 0),
        ];

        let filtered: Vec<Version> = filter_versions(versions.into_iter()).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], Version::f(2017, 3, 1, 0));
    }

    #[test]
    fn filters_highest_major_versions() {
        let versions = vec![
            Version::f(2017, 1, 1, 0),
            Version::f(2017, 2, 1, 0),
            Version::f(2017, 3, 1, 0),
            Version::f(2018, 1, 1, 0),
            Version::f(2018, 2, 1, 0),
            Version::f(2018, 3, 1, 0),
        ];

        let filtered: Vec<Version> = filter_versions(versions.into_iter()).collect();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&Version::f(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::f(2018, 3, 1, 0)));
    }

    #[test]
    fn filters_highest_from_each_release_type() {
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

        let filtered: Vec<Version> = filter_versions(versions.into_iter()).collect();
        assert_eq!(filtered.len(), 4);
        assert!(filtered.contains(&Version::a(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::b(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::p(2017, 3, 1, 0)));
        assert!(filtered.contains(&Version::f(2017, 3, 1, 0)));
    }

    #[test]
    fn filters_highest_from_major_and_each_release_type() {
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

        let filtered: Vec<Version> = filter_versions(versions.into_iter()).collect();
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
