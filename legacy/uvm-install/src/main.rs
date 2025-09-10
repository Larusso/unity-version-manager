use anyhow::Result;

use uvm_cli;
use uvm_install;

use console::style;
use log::info;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use uvm_core::unity::Component;
use uvm_core::Version;

use structopt::{
  clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};

const SETTINGS: &'static [AppSettings] = &[
  AppSettings::ColoredHelp,
  AppSettings::DontCollapseArgsInUsage,
];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
  /// The api version to install modules or editor
  version: Version,

  /// A directory to install the requested version to
  destination: Option<PathBuf>,

  /// install all support packages
  #[structopt(short, long)]
  all: bool,

  /// install android support for editor
  #[structopt(long)]
  android: bool,

  /// install ios support for editor
  #[structopt(long)]
  ios: bool,

  /// install webgl support for editor
  #[structopt(long)]
  webgl: bool,

  /// install mobile support (android, ios, webgl)
  #[structopt(long)]
  mobile: bool,

  /// install linux support for editor
  #[cfg(not(target_os = "linux"))]
  #[structopt(long)]
  linux: bool,

  /// install windows support for editor
  #[structopt(long)]
  #[cfg(not(target_os = "windows"))]
  windows: bool,

  /// install macos support for editor
  #[cfg(not(target_os = "macos"))]
  #[structopt(long)]
  macos: bool,

  /// install desktop support (linux, windows)
  #[structopt(long)]
  desktop: bool,

  /// skip installer verification
  #[structopt(long = "no-verify", parse(from_flag = std::ops::Not::not))]
  verify: bool,

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

impl Opts {
  #[cfg(not(target_os = "linux"))]
  fn linux(&self) -> bool {
    self.linux
  }

  #[cfg(target_os = "linux")]
  fn linux(&self) -> bool {
    false
  }

  #[cfg(not(target_os = "windows"))]
  fn windows(&self) -> bool {
    self.windows
  }
  #[cfg(target_os = "windows")]
  fn windows(&self) -> bool {
    false
  }

  #[cfg(not(target_os = "macos"))]
  fn macos(&self) -> bool {
    self.macos
  }

  #[cfg(target_os = "macos")]
  fn macos(&self) -> bool {
    false
  }
}

impl uvm_install::InstallerOptions for Opts {
  fn install_variants(&self) -> Option<HashSet<Component>> {
    if self.android
      || self.ios
      || self.webgl
      || self.linux()
      || self.windows()
      || self.macos()
      || self.mobile
      || self.desktop
      || self.all
    {
      let mut variants: HashSet<Component> = HashSet::with_capacity(5);

      if self.android || self.mobile || self.all {
        variants.insert(Component::Android);
      }

      if self.ios || self.mobile || self.all {
        variants.insert(Component::Ios);
      }

      if self.webgl || self.mobile || self.all {
        variants.insert(Component::WebGl);
      }

      if cfg!(not(target_os = "windows")) {
        if self.desktop || self.windows() || self.all {
          if self.version() >= &Version::from_str("2018.0.0b0").unwrap() {
            variants.insert(Component::WindowsMono);
          } else {
            variants.insert(Component::Windows);
          }
        }
      }

      if cfg!(not(target_os = "linux")) {
        if self.desktop || self.linux() || self.all {
          variants.insert(Component::Linux);
        }
      }

      if cfg!(not(target_os = "macos")) {
        if self.desktop || self.macos() || self.all {
          variants.insert(Component::MacMono);
        }
      }
      return Some(variants);
    }
    None
  }

  fn version(&self) -> &Version {
    &self.version
  }

  fn destination(&self) -> &Option<PathBuf> {
    &self.destination
  }

  fn skip_verification(&self) -> bool {
    !self.verify
  }
}

impl uvm_install::Options for Opts {
  fn verbose(&self) -> bool {
    self.verbose > 1
  }

  fn debug(&self) -> bool {
    self.debug
  }

  fn color(&self) -> &ColorOption {
    &self.color
  }
}

fn main() -> Result<()> {
  let opt = Opts::from_args();

  set_colors_enabled(&opt.color);
  set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

  uvm_install::UvmCommand::new()
    .exec(&opt)
    .unwrap_or_else(|err| {
      let message = "Failure during installation";
      eprintln!("{}", style(message).red());
      info!("{}", err);
      process::exit(1);
    });

  eprintln!("{}", style("Finish").green().bold());
  Ok(())
}
