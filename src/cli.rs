use docopt::Docopt;
use std::convert::From;
use std::str::FromStr;
use unity::Version;
use std::path::PathBuf;
use std::fmt;
use std::fmt::{Debug, Display};

#[derive(Debug, Deserialize)]
struct Arguments {}

#[derive(Debug, Deserialize)]
struct ListArguments {
    flag_verbose: bool,
}

#[derive(Debug, Deserialize)]
struct UseArguments {
    arg_version: String,
    flag_verbose: bool,
}

#[derive(Debug, Deserialize)]
struct LaunchArguments {
    arg_project_path: Option<PathBuf>,
    flag_platform: Option<UnityPlatform>,
    flag_force_project_version: bool,
    flag_verbose: bool,
}

#[derive(Debug, Deserialize)]
struct DetectArguments {
    arg_project_path: Option<PathBuf>,
    flag_verbose: bool,
}

#[derive(Debug)]
pub struct Options {}

#[derive(Debug)]
pub struct ListOptions {
    pub verbose: bool,
}

#[derive(Debug)]
pub struct UseOptions {
    pub version: Version,
    pub verbose: bool,
}

#[derive(Deserialize, Debug)]
pub enum UnityPlatform {
    Win32,
    Win64,
    OSX,
    Linux,
    Linux64,
    IOS,
    Android,
    Web,
    WebStreamed,
    WebGl,
    XboxOne,
    PS4,
    PSP2,
    WsaPlayer,
    Tizen,
    SamsungTV,
}

impl Display for UnityPlatform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let raw = format!("{:?}", self).to_lowercase();
        write!(f, "{}", raw)
    }
}

#[derive(Debug)]
pub struct LaunchOptions {
    pub project_path: Option<PathBuf>,
    pub platform: Option<UnityPlatform>,
    pub force_project_version: bool,
    pub verbose: bool
}

#[derive(Debug)]
pub struct DetectOptions {
    pub project_path: Option<PathBuf>,
    pub verbose: bool,
}

impl From<Arguments> for Options {
    fn from(_: Arguments) -> Self {
        Options {}
    }
}

impl From<ListArguments> for ListOptions {
    fn from(a: ListArguments) -> Self {
        ListOptions {
            verbose: a.flag_verbose,
        }
    }
}

impl From<UseArguments> for UseOptions {
    fn from(a: UseArguments) -> Self {
        UseOptions {
            verbose: a.flag_verbose,
            version: Version::from_str(&a.arg_version).expect("Can't read version parameter"),
        }
    }
}

impl From<LaunchArguments> for LaunchOptions {
    fn from(a: LaunchArguments) -> Self {
        LaunchOptions {
            verbose: a.flag_verbose,
            platform: a.flag_platform,
            force_project_version: a.flag_force_project_version,
            project_path: a.arg_project_path,
        }
    }
}

impl From<DetectArguments> for DetectOptions {
    fn from(a: DetectArguments) -> Self {
        DetectOptions {
            verbose: a.flag_verbose,
            project_path: a.arg_project_path,
        }
    }
}

pub fn get_use_options(usage: &str) -> Option<UseOptions> {
    let args: UseArguments = Docopt::new(usage)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    Some(args.into())
}

pub fn get_list_options(usage: &str) -> Option<ListOptions> {
    let args: ListArguments = Docopt::new(usage)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    Some(args.into())
}

pub fn get_launch_options(usage: &str) -> Option<LaunchOptions> {
    let args: LaunchArguments = Docopt::new(usage)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    Some(args.into())
}

pub fn get_detect_options(usage: &str) -> Option<DetectOptions> {
    let args: DetectArguments = Docopt::new(usage)
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    Some(args.into())
}

pub fn get_options(usage: &str) -> Option<Options> {
    let version = format!(
        "{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    );

    let args: Arguments = Docopt::new(usage)
        .and_then(|d| Ok(d.version(Some(version))))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    Some(args.into())
}
