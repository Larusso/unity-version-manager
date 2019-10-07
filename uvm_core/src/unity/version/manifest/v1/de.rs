pub mod bool {
    use serde::{Deserialize, Deserializer};
    use std::result;

    pub fn default() -> bool {
        false
    }

    pub fn deserialize<'de, D>(deserializer: D) -> result::Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(false)
        } else {
            match s.as_str() {
                "true" => Ok(true),
                "false" => Ok(true),
                _ => Ok(false),
            }
        }
    }
}
