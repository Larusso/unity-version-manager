use crate::sys::version as version_impl;
use derive_more::Display;
use nom::{
    branch::alt,
    character::complete::{char, digit1, hex_digit1, space1},
    combinator::{map_res, verify},
    error::context,
    sequence::delimited,
    IResult, Parser,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::{cmp::Ordering, str::FromStr, sync::OnceLock};
use regex::Regex;

mod release_type;
mod revision_hash;
use crate::error::VersionError;
pub use release_type::ReleaseType;
pub use revision_hash::RevisionHash;

#[derive(Eq, Debug, Clone, Hash, PartialOrd, Display)]
#[display("{}{}{}", base, release_type, revision)]
pub struct Version {
    base: semver::Version,
    release_type: ReleaseType,
    revision: u64,
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.base
            .cmp(&other.base)
            .then(self.release_type.cmp(&other.release_type))
            .then(self.revision.cmp(&other.revision))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
            && self.release_type == other.release_type
            && self.revision == other.revision
    }
}

impl AsRef<Version> for Version {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Version> for Version {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_version(s) {
            Ok((_, version)) => Ok(version),
            Err(err) => {
                let error_msg = match err {
                    nom::Err::Error(e) | nom::Err::Failure(e) => {
                        format!("Parse error at: {}", e.input)
                    },
                    _ => "Unknown parsing error".to_string(),
                };
                Err(VersionError::ParsingFailed(error_msg))
            }
        }
    }
}

impl TryFrom<&str> for Version {
    type Error = <Version as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Version::from_str(value)
    }
}

impl TryFrom<String> for Version {
    type Error = <Version as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Version::from_str(&value)
    }
}

impl TryFrom<PathBuf> for Version {
    type Error = VersionError;

    fn try_from(path: PathBuf) -> Result<Self, VersionError> {
        Version::from_path(path)
    }
}

impl TryFrom<&Path> for Version {
    type Error = VersionError;

    fn try_from(path: &Path) -> Result<Self, VersionError> {
        Version::from_path(path)
    }
}

impl Version {
    pub fn new(
        major: u64,
        minor: u64,
        patch: u64,
        release_type: ReleaseType,
        revision: u64,
    ) -> Version {
        let base = semver::Version::new(major, minor, patch);
        Version {
            base,
            release_type,
            revision,
        }
    }

    pub fn release_type(&self) -> ReleaseType {
        self.release_type
    }

    pub fn major(&self) -> u64 {
        self.base.major
    }

    pub fn minor(&self) -> u64 {
        self.base.minor
    }

    pub fn patch(&self) -> u64 {
        self.base.patch
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, VersionError> {
        version_impl::read_version_from_path(path)
    }

    pub fn from_string_containing<S: AsRef<str>>(s: S) -> Result<Self, VersionError> {
        static VERSION_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = VERSION_REGEX.get_or_init(|| {
            Regex::new(r"\b\d+\.\d+\.\d+[fabp]\d+\b").unwrap()
        });
        
        let s = s.as_ref();
        
        for mat in regex.find_iter(s) {
            if let Ok(version) = Version::from_str(mat.as_str()) {
                return Ok(version);
            }
        }
            Err(VersionError::ParsingFailed(format!("Could not find a valid Unity version in string: {}", s)))
    }

    pub fn base(&self) -> &semver::Version {
        &self.base
    }
}

#[derive(Eq, Debug, Clone, Hash, Display)]
#[display("{} ({})", version, revision)]
#[allow(dead_code)]
pub struct CompleteVersion {
    version: Version,
    revision: RevisionHash,
}

impl CompleteVersion {
    /// Creates a new CompleteVersion from a Version and RevisionHash.
    pub fn new(version: Version, revision: RevisionHash) -> Self {
        Self { version, revision }
    }
    
    /// Gets the version component.
    pub fn version(&self) -> &Version {
        &self.version
    }
    
    /// Gets the revision hash component.
    pub fn revision(&self) -> &RevisionHash {
        &self.revision
    }
}

impl FromStr for CompleteVersion {
    type Err = VersionError;

    /// Parses a complete Unity version string with revision hash.
    /// 
    /// # Format
    /// 
    /// Expects format: "VERSION (REVISION_HASH)"
    /// Examples: "2021.3.55f1 (f87d5274e360)", "2022.1.5f1 (abc123def456)"
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - No revision hash is present
    /// - Version format is invalid  
    /// - Revision hash format is invalid
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_complete_version(s.trim()) {
            Ok((remaining, complete_version)) => {
                if remaining.is_empty() {
                    Ok(complete_version)
                } else {
                    Err(VersionError::ParsingFailed(
                        format!("Unexpected remaining input: '{}'", remaining)
                    ))
                }
            }
            Err(err) => {
                let error_msg = match err {
                    nom::Err::Error(e) | nom::Err::Failure(e) => {
                        format!("Parse error at: '{}'", e.input)
                    }
                    nom::Err::Incomplete(_) => {
                        "Incomplete input".to_string()
                    }
                };
                Err(VersionError::ParsingFailed(error_msg))
            }
        }
    }
}

impl PartialEq for CompleteVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl PartialOrd for CompleteVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

fn parse_release_type(input: &str) -> IResult<&str, ReleaseType> {
    context(
        "release type",
        map_res(alt((char('f'), char('b'), char('a'), char('p'))), |c| {
            ReleaseType::try_from(c)
        }),
    ).parse(input)
}

fn parse_version(input: &str) -> IResult<&str, Version> {
    context(
        "version",
        (
            context("major version", map_res(digit1, |s: &str| s.parse::<u64>())),
            char('.'),
            context("minor version", map_res(digit1, |s: &str| s.parse::<u64>())),
            char('.'),
            context("patch version", map_res(digit1, |s: &str| s.parse::<u64>())),
            parse_release_type,
            context("revision", map_res(digit1, |s: &str| s.parse::<u64>())),
        )
    )
    .map(|(major, _, minor, _, patch, release_type, revision)| {
        let base = semver::Version::new(major, minor, patch);
        Version {
            base,
            release_type,
            revision,
        }
    })
    .parse(input)
}

fn parse_revision_hash(input: &str) -> IResult<&str, RevisionHash> {
    context(
        "revision hash",
        map_res(
            verify(hex_digit1, |s: &str| s.len() == 12),
            |hex_str: &str| RevisionHash::new(hex_str)
        )
    ).parse(input)
}

fn parse_complete_version(input: &str) -> IResult<&str, CompleteVersion> {
    context(
        "complete version",
        (
            parse_version,
            space1,
            delimited(char('('), parse_revision_hash, char(')')),
        )
    )
    .map(|(version, _, revision)| CompleteVersion::new(version, revision))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parse_version_string_with_valid_input() {
        let version_string = "1.2.3f4";
        let version = Version::from_str(version_string);
        assert!(version.is_ok(), "valid input returns a version")
    }

    #[test]
    fn splits_version_string_into_components() {
        let version_string = "11.2.3f4";
        let version = Version::from_str(version_string).unwrap();

        assert_eq!(version.base.major, 11, "parse correct major component");
        assert_eq!(version.base.minor, 2, "parse correct minor component");
        assert_eq!(version.base.patch, 3, "parse correct patch component");

        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 4, "parse correct revision component");
    }

    #[test]
    fn test_complete_version_from_str() {
        // Test successful parsing
        let complete_version = CompleteVersion::from_str("2021.3.55f1 (f87d5274e360)").unwrap();
        assert_eq!(complete_version.version().to_string(), "2021.3.55f1");
        assert_eq!(complete_version.revision().as_str(), "f87d5274e360");
        assert_eq!(complete_version.to_string(), "2021.3.55f1 (f87d5274e360)");

        // Test different version formats
        let alpha_version = CompleteVersion::from_str("2023.1.0a1 (123456789abc)").unwrap();
        assert_eq!(alpha_version.version().to_string(), "2023.1.0a1");
        assert_eq!(alpha_version.revision().as_str(), "123456789abc");

        // Test error cases with specific error message validation
        
        // No revision hash
        let no_hash_result = CompleteVersion::from_str("2021.3.55f1");
        assert!(no_hash_result.is_err());
        let error_msg = no_hash_result.unwrap_err().to_string();
        assert!(error_msg.contains("Parse error"), "Expected parsing error for missing hash, got: {}", error_msg);
        
        // Invalid version format
        let invalid_version_result = CompleteVersion::from_str("invalid (f87d5274e360)");
        assert!(invalid_version_result.is_err());
        let error_msg = invalid_version_result.unwrap_err().to_string();
        assert!(error_msg.contains("Parse error"), "Expected parsing error for invalid version, got: {}", error_msg);
        
        // Invalid hash characters
        let invalid_hash_result = CompleteVersion::from_str("2021.3.55f1 (invalid)");
        assert!(invalid_hash_result.is_err());
        let error_msg = invalid_hash_result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid revision hash") || error_msg.contains("Parse error"), 
                "Expected revision hash error, got: {}", error_msg);
        
        // Hash too short
        let short_hash_result = CompleteVersion::from_str("2021.3.55f1 (f87d527)");
        assert!(short_hash_result.is_err());
        let error_msg = short_hash_result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid revision hash") || error_msg.contains("Parse error"), 
                "Expected revision hash error for short hash, got: {}", error_msg);
        
        // Hash too long
        let long_hash_result = CompleteVersion::from_str("2021.3.55f1 (f87d5274e360ab)");
        assert!(long_hash_result.is_err());
        let error_msg = long_hash_result.unwrap_err().to_string();
        assert!(error_msg.contains("Parse error"), "Expected parsing error for long hash, got: {}", error_msg);
        
        // Non-hex characters in hash
        let non_hex_result = CompleteVersion::from_str("2021.3.55f1 (f87d5274e36z)");
        assert!(non_hex_result.is_err());
        let error_msg = non_hex_result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid revision hash") || error_msg.contains("Parse error"), 
                "Expected revision hash error for non-hex chars, got: {}", error_msg);
    }

    #[test]
    fn extracts_version_from_text() {
        let text = "Some text before 2023.1.4f5 and some after";
        let result = Version::from_string_containing(text);
        assert!(result.is_ok(), "Should successfully extract the version");

        let version = result.unwrap();
        assert_eq!(version.base.major, 2023);
        assert_eq!(version.base.minor, 1);
        assert_eq!(version.base.patch, 4);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 5);
    }


    #[test]
    fn extracts_version_from_text_and_returns_first_complete_version() {
        let text = "Some text 23 before 2023.1.4f5 and some after";
        let result = Version::from_string_containing(text);
        assert!(result.is_ok(), "Should successfully extract the version");

        let version = result.unwrap();
        assert_eq!(version.base.major, 2023);
        assert_eq!(version.base.minor, 1);
        assert_eq!(version.base.patch, 4);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 5);
    }

    proptest! {
        #[test]
        fn from_str_does_not_crash(s in "\\PC*") {
            let _v = Version::from_str(&s);
        }

        #[test]
        fn from_str_supports_all_valid_cases(
            major in 0u64..=u64::MAX,
            minor in 0u64..=u64::MAX,
            patch in 0u64..=u64::MAX,
            release_type in prop_oneof!["f", "p", "b", "a"],
            revision in 0u64..=u64::MAX,
        ) {
            let version_string = format!("{}.{}.{}{}{}", major, minor, patch, release_type, revision);
            let version = Version::from_str(&version_string).unwrap();

            assert!(version.base.major == major, "parse correct major component");
            assert!(version.base.minor == minor, "parse correct minor component");
            assert!(version.base.patch == patch, "parse correct patch component");

            assert_eq!(version.release_type, ReleaseType::from_str(&release_type).unwrap());
            assert!(version.revision == revision, "parse correct revision component");
        }
    }
}
