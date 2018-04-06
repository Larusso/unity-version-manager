use super::ColorOption;
use uvm_core::unity::Version;

#[derive(Debug, Deserialize)]
pub struct UseOptions {
    #[serde(with = "unity_version_format")]
    arg_version: Version,
    flag_verbose: bool,
    flag_color: ColorOption
}

impl UseOptions {
    pub fn version(&self) -> &Version {
        &self.arg_version
    }
}

mod unity_version_format {
    use uvm_core::unity::Version;
    use std::str::FromStr;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl super::Options for UseOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}
