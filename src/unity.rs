use regex::Regex;
use std::fmt;
use std::str::FromStr;

#[derive(PartialEq,Debug)]
pub enum VersionType {
    Final,
    Patch,
    Beta,
}

#[derive(PartialEq,Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    release_type: VersionType,
    revision: u32,
}

impl Version {
    pub fn new() -> Version {
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            release_type: VersionType::Final,
            revision: 0,
        }
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

pub struct ParseVersionError {}

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match read_unity_version(s) {
            Some(v) => Ok(v),
            None => Err(ParseVersionError{})
        }
    }
}

fn read_unity_version(version_string: &str) -> Option<Version> {
    let version_pattern = Regex::new(r"([0-9]{1,4})\.([0-9]{1,4})\.([0-9]{1,4})(f|p|b)([0-9]{1,4})").unwrap();
    match version_pattern.captures(version_string) {
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
            Some(Version {
                major,
                minor,
                patch,
                revision,
                release_type: release_type.unwrap(),
            })
        }
        None => None,
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

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {
            Version::from_str(s);
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

    invalid_version_input! {
        when_version_is_empty: "dsd",
        when_version_is_a_random_string: "sdfrersdfgsdf",
        when_version_is_a_short_version: "1.2",
        when_version_is_semver: "1.2.3",
        when_version_contains_unknown_release_type: "1.2.3g2"
    }

    valid_version_input! {
        when_version_has_single_digits: "1.2.3f4",
        when_version_has_long_digits: "0.0.0f43"
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
}
