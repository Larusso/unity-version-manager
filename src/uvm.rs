#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate regex;

pub mod cli;
pub mod cmd;
mod utils;

#[macro_export]
macro_rules! cargo_version {
    // `()` indicates that the macro takes no argument.
    () => (
        // The macro will expand into the contents of this block.
        format!("{}.{}.{}{}",
          env!("CARGO_PKG_VERSION_MAJOR"),
          env!("CARGO_PKG_VERSION_MINOR"),
          env!("CARGO_PKG_VERSION_PATCH"),
          option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""));
    )
}
