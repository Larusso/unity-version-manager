use std::str::FromStr;
use unity::Version;

#[derive(Debug, Deserialize)]
pub struct UseOptions {
    arg_version: String,
    flag_verbose: bool,
}

impl UseOptions {
    pub fn version(&self) -> Version {
        Version::from_str(&self.arg_version).expect("Can't read version parameter")
    }

    pub fn verbose(&self) -> bool {
        self.flag_verbose
    }
}
