use super::error::ParseComponentError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

use Category::*;
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum Category {
    DevTools,
    Plugins,
    Documentation,
    Components,
    Platforms,
    LanguagePack,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DevTools => "Dev tools",
            Plugins => "Plugins",
            Documentation => "Documentation",
            Components => "Components",
            Platforms => "Platforms",
            LanguagePack => "Language packs (Preview)",
        };
        write!(f, "{}", s)
    }
}

impl Default for Category {
    fn default() -> Self {
        Platforms
    }
}

impl FromStr for Category {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DevTools" | "Dev tools" => Ok(DevTools),
            "Plugins" => Ok(Plugins),
            "Documentation" => Ok(Documentation),
            "Components" => Ok(Components),
            "Platforms" => Ok(Platforms),
            "LanguagePack" | "Language packs (Preview)" => Ok(LanguagePack),
            x => Err(ParseComponentError::UnsupportedCategory(x.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Category::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
