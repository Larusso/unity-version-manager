use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use structopt::{
    clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};
use uvm_core::unity::VersionType;
use uvm_versions;

const SETTINGS: &'static [AppSettings] = &[
    AppSettings::ColoredHelp,
    AppSettings::DontCollapseArgsInUsage,
];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    /// list all available versions for the selected version types
    #[structopt(short, long)]
    all: bool,

    /// list final versions
    #[structopt(short = "f", long = "final")]
    list_final: bool,

    /// list beta versions
    #[structopt(short = "b", long = "beta")]
    list_beta: bool,

    /// list alpha versions
    #[structopt(long = "alpha")]
    list_alpha: bool,

    /// list patch versions
    #[structopt(short = "p", long = "patch")]
    list_patch: bool,

    /// a regex pattern to filter results
    #[structopt()]
    pattern: Option<Regex>,

    /// print debug output
    #[structopt(short, long)]
    debug: bool,

    /// print more output
    #[structopt(short, long, parse(from_occurrences))]
    verbose: i32,

    /// Color:.
    #[structopt(short, long, possible_values = &ColorOption::variants(), case_insensitive = true, default_value)]
    color: ColorOption,
}

impl uvm_versions::VersionListOptions for Opts {
    fn list_variants(&self) -> HashSet<VersionType> {
        let mut variants: HashSet<VersionType> = HashSet::with_capacity(4);

        if self.has_variant_flags() {
            if self.list_alpha {
                variants.insert(VersionType::Alpha);
            }

            if self.list_beta {
                variants.insert(VersionType::Beta);
            }

            if self.list_patch {
                variants.insert(VersionType::Patch);
            }

            if self.list_final {
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
        self.list_alpha || self.list_beta || self.list_patch || self.list_final
    }

    fn filter_versions(&self) -> bool {
        !self.all
    }

    fn pattern(&self) -> &Option<Regex> {
        &self.pattern
    }

    fn debug(&self) -> bool {
        self.debug
    }
}

fn main() -> Result<()> {
    let opt = Opts::from_args();

    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

    uvm_versions::list_versions(&opt).context("failed to list Unity modules")?;
    Ok(())
}
