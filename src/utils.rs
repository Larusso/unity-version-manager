use regex::Regex;
use std::fmt;

#[derive(PartialEq,Debug)]
pub enum VersionType {
    Final,
    Patch,
    Beta,
}

#[derive(Debug)]
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

pub fn read_unity_version(version_string: &str) -> Option<Version> {
    let version_pattern = Regex::new(r"(\d+)\.(\d+)\.(\d+)(f|p|b)(\d+)").unwrap();
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
                    let version = read_unity_version(version_string);
                    assert!(version.is_none(), "invalid input returns None")
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

    #[test]
    fn parse_version_string_with_invalid_input() {
        let version_string = "";
        let version = read_unity_version(version_string);
        assert!(version.is_none(), "invalid input returns no version")
    }

    #[test]
    fn parse_version_string_with_valid_input() {
        let version_string = "1.2.3f4";
        let version = read_unity_version(version_string);
        assert!(version.is_some(), "valid input returns a version")
    }

    #[test]
    fn splits_version_string_into_components() {
        let version_string = "1.2.3f4";
        let version = read_unity_version(version_string).unwrap();

        assert!(version.major == 1, "parse correct major component");

        assert!(version.minor == 2, "parse correct minor component");

        assert!(version.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, VersionType::Final);

        assert!(version.revision == 4, "parse correct revision component");
    }

    #[test]
    fn splits_patch_version_string_into_components() {
        let version_string = "1.2.3p4";
        let version = read_unity_version(version_string).unwrap();

        assert!(version.major == 1, "parse correct major component");

        assert!(version.minor == 2, "parse correct minor component");

        assert!(version.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, VersionType::Patch);

        assert!(version.revision == 4, "parse correct revision component");
    }

    #[test]
    fn splits_beta_version_string_into_components() {
        let version_string = "1.2.3b4";
        let version = read_unity_version(version_string).unwrap();

        assert!(version.major == 1, "parse correct major component");

        assert!(version.minor == 2, "parse correct minor component");

        assert!(version.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, VersionType::Beta);

        assert!(version.revision == 4, "parse correct revision component");
    }

    #[test]
    fn splitted_version_final_can_be_converted_back_to_string() {
        let version_string = "1.2.3f4";
        let version = read_unity_version(version_string).unwrap();

        assert_eq!(version.to_string(), version_string)
    }

    #[test]
    fn splitted_version_patch_can_be_converted_back_to_string() {
        let version_string = "4.6.12p5";
        let version = read_unity_version(version_string).unwrap();

        assert_eq!(version.to_string(), version_string)
    }

    #[test]
    fn splitted_version_beta_can_be_converted_back_to_string() {
        let version_string = "2017.2.1b1";
        let version = read_unity_version(version_string).unwrap();

        assert_eq!(version.to_string(), version_string)
    }
}
