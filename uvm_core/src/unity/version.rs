use regex::Regex;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use std::error::Error;
use std::result;
use std::convert::From;
use unity::Installation;
use serde;

#[derive(PartialEq,Eq,Ord,Hash,Debug)]
pub enum VersionType {
    Beta,
    Patch,
    Final,
}

impl PartialOrd for VersionType {
    fn partial_cmp(&self, other: &VersionType) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq,Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    release_type: VersionType,
    revision: u32,
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.major.cmp(&other.major)
        .then(self.minor.cmp(&other.minor))
        .then(self.patch.cmp(&other.patch))
        .then(self.release_type.cmp(&other.release_type))
        .then(self.revision.cmp(&other.revision))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        (self.release_type == other.release_type)
        && (self.major == other.major)
        && (self.minor == other.minor)
        && (self.patch == other.patch)
        && (self.revision == other.revision)
    }
}

impl Version {
    pub fn release_type(&self) -> &VersionType {
        &self.release_type
    }
}

impl fmt::Display for VersionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &VersionType::Final => write!(f, "f"),
            &VersionType::Patch => write!(f, "p"),
            &VersionType::Beta => write!(f, "b"),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}{}{}",
            self.major, self.minor, self.patch, self.release_type.to_string(), self.revision
        )
    }
}

#[derive(Debug)]
pub struct ParseVersionError {
    message: String
}

impl ParseVersionError {
    fn new(message: &str) -> ParseVersionError {
        ParseVersionError { message: String::from(message) }
    }
}

impl fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseVersionError")
    }
}

impl Error for ParseVersionError {
    fn description(&self) -> &str {
        &self.message[..]
    }
}

pub type Result<T> = result::Result<T, ParseVersionError>;

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self> {
        let version_pattern = Regex::new(r"([0-9]{1,4})\.([0-9]{1,4})\.([0-9]{1,4})(f|p|b)([0-9]{1,4})").unwrap();
        match version_pattern.captures(s) {
            Some(caps) => {
                let major: u32 = caps.get(1).map_or("0", |m| m.as_str()).parse().unwrap();
                let minor: u32 = caps.get(2).map_or("0", |m| m.as_str()).parse().unwrap();
                let patch: u32 = caps.get(3).map_or("0", |m| m.as_str()).parse().unwrap();

                let release_type = match caps.get(4).map_or("", |m| m.as_str()) {
                    "f" => Some(VersionType::Final),
                    "p" => Some(VersionType::Patch),
                    "b" => Some(VersionType::Beta),
                    _ => None,
                };

                let revision: u32 = caps.get(5).map_or("0", |m| m.as_str()).parse().unwrap();
                Ok(Version {
                    major,
                    minor,
                    patch,
                    revision,
                    release_type: release_type.unwrap(),
                })
            }
            None => Err( ParseVersionError::new("Failed to match version pattern to input") ),
        }
    }
}

impl From<Installation> for Version {
    fn from(item: Installation) -> Self {
        item.version_owned()
    }
}

pub mod unity_version_format {
    use super::Version;
    use std::str::FromStr;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! invalid_version_input {
        ($($name:ident: $input:expr),*) => {
            $(
                #[test]
                fn $name() {
                    let version_string = $input;
                    let version = Version::from_str(version_string);
                    assert!(version.is_err(), "invalid input returns None")
                }
            )*
        };
    }

    macro_rules! valid_version_input {
        ($($name:ident: $input:expr),*) => {
            $(
                #[test]
                fn $name() {
                    let version_string = $input;
                    let version = Version::from_str(version_string);
                    assert!(version.is_ok(), "valid input returns a version")
                }
            )*
        };
    }

    invalid_version_input! {
        when_version_is_empty: "dsd",
        when_version_is_a_random_string: "sdfrersdfgsdf",
        when_version_is_a_short_version: "1.2",
        when_version_is_semver: "1.2.3",
        when_version_contains_unknown_release_type: "1.2.3g2"
    }

    valid_version_input! {
        when_version_has_single_digits: "1.2.3f4",
        when_version_has_long_digits: "0.0.0f43",
        when_version_has_only_zero_digits: "0.0.0f0"
    }

    #[test]
    fn parse_version_string_with_valid_input() {
        let version_string = "1.2.3f4";
        let version = Version::from_str(version_string);
        assert!(version.is_ok(), "valid input returns a version")
    }

    #[test]
    fn splits_version_string_into_components() {
        let version_string = "1.2.3f4";
        let version = Version::from_str(version_string).ok().unwrap();

        assert!(version.major == 1, "parse correct major component");

        assert!(version.minor == 2, "parse correct minor component");

        assert!(version.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, VersionType::Final);

        assert!(version.revision == 4, "parse correct revision component");
    }

    #[test]
    fn orders_version_final_release_greater_than_patch() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("1.2.3p4").ok().unwrap();
        assert_eq!(Ordering::Greater, version_a.cmp(&version_b));
    }

    #[test]
    fn orders_version_patch_release_greater_than_beta() {
        let version_a = Version::from_str("1.2.3p4").ok().unwrap();
        let version_b = Version::from_str("1.2.3b4").ok().unwrap();
        assert_eq!(Ordering::Greater, version_a.cmp(&version_b));
    }

    #[test]
    fn orders_version_final_release_greater_than_beta() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("1.2.3b4").ok().unwrap();
        assert_eq!(Ordering::Greater, version_a.cmp(&version_b));
    }

    #[test]
    fn orders_version_all_equak() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("1.2.3f4").ok().unwrap();
        assert_eq!(Ordering::Equal, version_a.cmp(&version_b));
    }

    #[test]
    fn orders_version_major_smaller() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("0.2.3f4").ok().unwrap();
        assert_eq!(Ordering::Greater, version_a.cmp(&version_b));
    }

    #[test]
    fn orders_version_minor_smaller() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("1.1.3f4").ok().unwrap();
        assert_eq!(Ordering::Greater, version_a.cmp(&version_b));
    }

    #[test]
    fn orders_version_patch_smaller() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("1.2.2f4").ok().unwrap();
        assert_eq!(Ordering::Greater, version_a.cmp(&version_b));
    }

    #[test]
    fn orders_version_revision_smaller() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("1.2.3f3").ok().unwrap();
        assert_eq!(Ordering::Greater, version_a.cmp(&version_b));
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {
            Version::from_str(s).is_ok();
        }

        #[test]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            Version::from_str(s).ok().unwrap();
        }

        #[test]
        fn parses_version_back_to_original(major in 0u32..9999, minor in 0u32..9999, patch in 0u32..9999, revision in 0u32..9999 ) {
            let v1 = Version {
                major,
                minor,
                patch,
                revision,
                release_type: VersionType::Final};

            let v2 = Version::from_str(&format!("{:04}.{:04}.{:04}f{:04}", major, minor, patch, revision)).ok().unwrap();
            prop_assert_eq!(v1, v2);
        }
    }
}
