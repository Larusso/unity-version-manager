use console::style;

use console::Term;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use semver::VersionReq;
use serde::{Deserialize, Deserializer};
use std::io;
use std::result;
use uvm_cli::ColorOption;
use uvm_core::unity::VersionType;
use log::{debug,info};
use uvm_core::unity::fetch_matching_version;

#[derive(Debug, Deserialize)]
pub struct FetchOptions {
    #[serde(deserialize_with = "deserialize_semver")]
    arg_version_req: VersionReq,
    arg_release_type: Option<VersionType>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_color: ColorOption,
}

fn has_operand(ranges: &str) -> bool {
    ranges.starts_with("^")
        || ranges.starts_with("~")
        || ranges.starts_with(">")
        || ranges.starts_with("<")
        || ranges.starts_with("=")
}

fn deserialize_semver<'de, D>(deserializer: D) -> result::Result<VersionReq, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;
    if s.is_empty() {
        s.push('*');
    } else if !has_operand(s.as_str().trim()) {
        s.insert(0, '~');
    }

    VersionReq::parse(&s).map_err(serde::de::Error::custom)
}

impl FetchOptions {
    fn version_req(&self) -> VersionReq {
        self.arg_version_req.clone()
    }

    fn release_type(&self) -> VersionType {
        self.arg_release_type
            .unwrap_or_else(|| VersionType::default())
    }
}

impl uvm_cli::Options for FetchOptions {
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

pub fn exec(options: &FetchOptions) -> io::Result<()> {
    let stdout = Term::stderr();
    let progress = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    progress.set_style(spinner_style);
    progress.set_draw_target(progress_draw_target(options));
    progress.set_prefix("search unity version");
    progress.enable_steady_tick(100);
    progress.tick();

    debug!("fetch versions list");
    let versions = uvm_core::unity::all_versions()?;
    progress.finish_and_clear();

    let version = fetch_matching_version(versions, options.version_req(), options.release_type())?;
    info!("highest matching version:");
    stdout.write_line(&format!("{}", style(version).green().bold()))
}
