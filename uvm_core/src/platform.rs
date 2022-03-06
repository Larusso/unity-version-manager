use std::fmt::{self, Display};

#[derive(Debug, Deserialize, Clone, Copy, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[serde(alias = "osx")]
    MacOs,
    Linux,
    Win,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(windows) {
            Platform::Win
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            panic!("uvm doens't compile for this platform")
        }
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Platform::MacOs => write!(f, "osx"),
            Platform::Linux => write!(f, "linux"),
            Platform::Win => write!(f, "win"),
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::current()
    }
}

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ParsePlatformError {
        #[error("platform not supported: {0}")]
        NotSupported(String)
    }
}

impl std::str::FromStr for Platform {
    type Err = self::error::ParsePlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "osx" => Ok(Platform::MacOs),
            "linux" => Ok(Platform::Linux),
            "win" => Ok(Platform::Win),
            x => Err(self::error::ParsePlatformError::NotSupported(x.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_creates_sytem_default_platform() {
        let system_platform = if cfg!(windows) {
            Platform::Win
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            panic!("uvm doens't compile for this platform")
        };

        assert_eq!(system_platform, Platform::default());
    }

    #[test]
    fn can_be_printed_as_string() {
        assert_eq!(&format!("{}",Platform::MacOs), "osx");
        assert_eq!(&format!("{}",Platform::Win), "win");
        assert_eq!(&format!("{}",Platform::Linux), "linux");
    }

    #[derive(Debug, Deserialize)]
    struct TestData {
        field: Platform,
    }

    #[test]
    fn macos_can_be_deserialized() {
        let data = r#"
        {
            "field": "macos"
        }"#;

        let test:TestData = serde_json::from_str(data).unwrap();
        assert_eq!(test.field, Platform::MacOs);
    }

    #[test]
    fn win_can_be_deserialized() {
        let data = r#"
        {
            "field": "win"
        }"#;

        let test:TestData = serde_json::from_str(data).unwrap();
        assert_eq!(test.field, Platform::Win);
    }

    #[test]
    fn linux_can_be_deserialized() {
        let data = r#"
        {
            "field": "linux"
        }"#;

        let test:TestData = serde_json::from_str(data).unwrap();
        assert_eq!(test.field, Platform::Linux);
    }

    #[test]
    fn osx_can_be_deserialized() {
        let data = r#"
        {
            "field": "osx"
        }"#;

        let test:TestData = serde_json::from_str(data).unwrap();
        assert_eq!(test.field, Platform::MacOs);
    }
}
