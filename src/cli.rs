use docopt::Docopt;
use std::convert::From;

#[derive(Debug, Deserialize)]
struct Arguments {}

#[derive(Debug, Deserialize)]
struct ListArguments {
    flag_verbose: bool,
}

#[derive(Debug)]
pub struct Options {}

#[derive(Debug)]
pub struct ListOptions {
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

pub fn get_list_options(usage: &str) -> Option<ListOptions> {
    let args: ListArguments = Docopt::new(usage)
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
