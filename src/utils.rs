use regex::Regex;

pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    revision: u32,
}

impl Version {
    pub fn new() -> Version {
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            revision: 0,
        }
    }
}

pub fn read_unity_version(version_string: &str) -> Option<Version> {
    let version_pattern = Regex::new(r"((\d+)\.(\d+)\.(\d+)((f|p|b)(\d+))?)$").unwrap();
    match version_pattern.captures(version_string) {
        Some(caps) => {
            let major: u32 = caps.get(2).map_or("0", |m| m.as_str()).parse().unwrap();
            let minor: u32 = caps.get(3).map_or("0", |m| m.as_str()).parse().unwrap();
            let patch: u32 = caps.get(4).map_or("0", |m| m.as_str()).parse().unwrap();
            //let r_type = caps.get(6).unwrap();
            let revision: u32 = caps.get(7).map_or("0", |m| m.as_str()).parse().unwrap();
            Some(Version {
                major,minor,patch, revision
            })
        },
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_string_with_invalid_input() {
        let version_string = "";
        let version = read_unity_version(version_string);
        assert!(
            version.is_none(),
            "invalid input returns no version"
        )
    }

    #[test]
    fn parse_version_string_with_valid_input() {
        let version_string = "1.2.3f4";
        let version = read_unity_version(version_string);
        assert!(
            version.is_some(),
            "valid input returns a version"
        )
    }

    #[test]
    fn splits_version_string_into_components() {
        let version_string = "1.2.3f4";
        let version = read_unity_version(version_string).unwrap();

        assert!(
            version.major == 1,
            "parse correct major component"
        );

        assert!(
            version.minor == 2,
            "parse correct minor component"
        );

        assert!(
            version.patch == 3,
            "parse correct patch component"
        );

        assert!(
            version.revision == 4,
            "parse correct revision component"
        );
    }
}
