use std::str::FromStr;
use unity::Version;
use serde;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UseOptions {
    #[serde(with = "unity_version_format")]
    arg_version: Version,
    flag_verbose: bool,
}

impl UseOptions {
    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn verbose(&self) -> bool {
        self.flag_verbose
    }
}

mod unity_version_format {
    use unity::Version;
    use std::str::FromStr;
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}
