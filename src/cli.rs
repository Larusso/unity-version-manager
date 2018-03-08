use docopt::Docopt;
use std::convert::From;

#[derive(Debug, Deserialize)]
struct Arguments {
}

#[derive(Debug)]
pub struct Options {
}

impl From<Arguments> for Options {
    fn from(_: Arguments) -> Self {
        Options {}
    }
}

pub fn get_options(usage: &str) -> Option<Options> {
    let version = format!("{}.{}.{}{}",
                     env!("CARGO_PKG_VERSION_MAJOR"),
                     env!("CARGO_PKG_VERSION_MINOR"),
                     env!("CARGO_PKG_VERSION_PATCH"),
                     option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));

    let args: Arguments = Docopt::new(usage)
                              .and_then(|d| Ok(d.version(Some(version))))
                              .and_then(|d| d.deserialize())
                              .unwrap_or_else(|e| e.exit());
    Some(args.into())
}