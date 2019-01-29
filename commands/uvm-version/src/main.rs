use anyhow::Result;
use console::style;
use structopt::{clap::crate_authors, clap::crate_description, clap::crate_version, StructOpt};
use uvm_cli;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use semver::VersionReq;
use log::*;
use uvm_core::unity::VersionType;
use uvm_core::unity::fetch_matching_version;
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};

fn has_operand(ranges: &str) -> bool {
  ranges.starts_with("^")
      || ranges.starts_with("~")
      || ranges.starts_with(">")
      || ranges.starts_with("<")
      || ranges.starts_with("=")
}

fn version_req(s: &str) -> Result<VersionReq, &'static str> {
  let mut s = String::from(s);
  if s.is_empty() {
    s.push('*');
  } else if !has_operand(s.trim()) {
    s.insert(0, '~');
  }
  VersionReq::parse(&s).map_err(|_| "expected semver version req") 
}

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
  /// print more output
  #[structopt(short, long, parse(from_occurrences))]
  verbose: i32,

  /// print debug output
  #[structopt(short, long)]
  debug: bool,

  /// Color:.
  #[structopt(short, long, possible_values = &ColorOption::variants(), case_insensitive = true, default_value)]
  color: ColorOption,

  #[structopt(subcommand)]
  cmd: Command
}

#[derive(StructOpt, Debug)]
enum Command {
  /// Return version matching version req.
  Matching {
    /// The version requirement string
    /// 
    /// The version requirement string will be converted to `semver::VersionReq`
    /// See https://docs.rs/semver/1.0.2/semver/struct.VersionReq.html for usage.
    #[structopt(parse(try_from_str = version_req))]
    version_req: VersionReq,

    /// The unity release type
    /// 
    /// The release type to limit the search for.
    #[structopt(possible_values=&["f", "final","p", "patch","b", "beta","a", "alpha"], case_insensitive=false, default_value)]
    release_type: VersionType,
  },
  Latest {
    #[structopt(possible_values=&["f", "final","p", "patch","b", "beta","a", "alpha"], case_insensitive=false, default_value)]
    release_type: VersionType,
  }
}


fn progress_draw_target(options: &Opts) -> ProgressDrawTarget {
    if options.debug {
        ProgressDrawTarget::hidden()
    } else {
        ProgressDrawTarget::stderr()
    }
}

fn main() -> Result<()> {
  let opt = Opts::from_args_safe().map(|opt| {
          set_colors_enabled(&opt.color);
          set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));
          opt
    })?; 

    let progress = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    progress.set_style(spinner_style);
    progress.set_draw_target(progress_draw_target(&opt));
    progress.set_prefix("search unity version");
    progress.enable_steady_tick(100);
    progress.tick();

    debug!("fetch versions list");
    let versions = uvm_core::unity::all_versions()?;
    progress.finish_and_clear();

    let (version_req, version_type) = match opt.cmd {
      Command::Latest { release_type } => (VersionReq::parse("*").unwrap(), release_type),
      Command::Matching { version_req, release_type } => (version_req, release_type),
    };

    let version = fetch_matching_version(versions, version_req, version_type)?;
    info!("highest matching version:");
    eprintln!("{}", style(version).green().bold());
    Ok(())

}
