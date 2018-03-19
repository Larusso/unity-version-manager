#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate regex;

#[cfg(test)]
#[macro_use]
extern crate proptest;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate tempdir;

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

pub mod cli;
pub mod unity;

pub use self::unity::list_installations;
pub use self::unity::current_installation;
pub use self::unity::Installation;
pub use self::unity::CurrentInstallation;
pub use self::unity::Version;
