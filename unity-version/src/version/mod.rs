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
use std::{cmp::Ordering, str::FromStr};
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
        let s = s.as_ref();
        Self::extract_version_from_text(s)
            .ok_or_else(|| VersionError::ParsingFailed(format!("Could not find a valid Unity version in string: {}", s)))
    }

    /// Extract Unity version from text using prioritized approach.
    /// Prioritizes versions with hashes (more reliable) over standalone versions.
    fn extract_version_from_text(text: &str) -> Option<Version> {
        use std::sync::OnceLock;
        
        // Enhanced regex to capture versions with optional hash suffixes
        static VERSION_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = VERSION_REGEX.get_or_init(|| {
            Regex::new(r"([0-9]{1,4})\.([0-9]{1,4})\.([0-9]{1,4})(f|p|b|a)([0-9]{1,4})(_([a-z0-9]{12})| \(([a-z0-9]{12})\)|/([a-z0-9]{12}))?").unwrap()
        });
        
        // Priority 1: Look for versions with parentheses hash format (most authoritative)
        for captures in regex.captures_iter(text) {
            if captures.get(8).is_some() {
                // This version has a hash in parentheses format: "version (hash)"
                let version_string = format!(
                    "{}.{}.{}{}{}",
                    &captures[1], &captures[2], &captures[3], &captures[4], &captures[5]
                );
                
                if let Ok(version) = Version::from_str(&version_string) {
                    return Some(version);
                }
            }
        }
        
        // Priority 2: Look for versions with underscore hash format
        for captures in regex.captures_iter(text) {
            if captures.get(7).is_some() {
                // This version has a hash in underscore format
                let version_string = format!(
                    "{}.{}.{}{}{}",
                    &captures[1], &captures[2], &captures[3], &captures[4], &captures[5]
                );
                
                if let Ok(version) = Version::from_str(&version_string) {
                    return Some(version);
                }
            }
        }
        
        // Priority 3: Look for versions with slash hash format
        for captures in regex.captures_iter(text) {
            if captures.get(9).is_some() {
                // This version has a hash in slash format
                let version_string = format!(
                    "{}.{}.{}{}{}",
                    &captures[1], &captures[2], &captures[3], &captures[4], &captures[5]
                );
                
                if let Ok(version) = Version::from_str(&version_string) {
                    return Some(version);
                }
            }
        }
        
        // Priority 4: Fallback to any version string found (without hash requirement)
        for captures in regex.captures_iter(text) {
            let version_string = format!(
                "{}.{}.{}{}{}",
                &captures[1], &captures[2], &captures[3], &captures[4], &captures[5]
            );
            
            if let Ok(version) = Version::from_str(&version_string) {
                return Some(version);
            }
        }
        
        None
    }

    pub fn base(&self) -> &semver::Version {
        &self.base
    }

    /// Find Unity version by running `strings` on an executable and parsing the output.
    /// This works on Unix-like systems (Linux, macOS) where the `strings` command is available.
    #[cfg(unix)]
    pub fn find_version_in_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, VersionError> {
        use std::process::{Command, Stdio};
        use log::debug;

        let path = path.as_ref();
        debug!("find api version in Unity executable {}", path.display());

        let child = Command::new("strings")
            .arg("--")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| VersionError::Other {
                source: e.into(),
                msg: "failed to spawn strings".to_string(),
            })?;

        let output = child.wait_with_output().map_err(|e| VersionError::Other {
            source: e.into(),
            msg: "failed to spawn strings".to_string(),
        })?;

        if !output.status.success() {
            return Err(VersionError::ExecutableContainsNoVersion(
                path.to_path_buf(),
            ));
        }

        let strings_output = String::from_utf8_lossy(&output.stdout);
        
        // Use the shared version extraction logic
        Self::extract_version_from_text(&strings_output)
            .map(|version| {
                debug!("found version {} in executable", &version);
                version
            })
            .ok_or_else(|| VersionError::ExecutableContainsNoVersion(path.to_path_buf()))
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
    /// Generate test data with bogus content and Unity versions in various positions
    fn generate_test_data_with_versions(
        parentheses_version: Option<&str>,
        underscore_version: Option<&str>, 
        slash_version: Option<&str>,
        standalone_versions: &[&str],
        bogus_data_size: usize
    ) -> String {
        let mut content = String::new();
        
        // Add initial bogus data
        for i in 0..bogus_data_size / 4 {
            content.push_str(&format!("__libc_start_main_{}\n", i));
            content.push_str("malloc\nfree\nstrlen\n");
            content.push_str("/lib64/ld-linux-x86-64.so.2\n");
            content.push_str("Some random binary string data\n");
        }
        
        // Add standalone versions scattered throughout
        for (idx, version) in standalone_versions.iter().enumerate() {
            if idx % 2 == 0 {
                content.push_str(&format!("Random text {}\n", idx));
            }
            content.push_str(&format!("{}\n", version));
            content.push_str("More random data\n");
        }
        
        // Add more bogus data
        for i in 0..bogus_data_size / 4 {
            content.push_str(&format!("function_name_{}\n", i));
            content.push_str("symbol_table_entry\n");
            content.push_str("debug_info_string\n");
        }
        
        // Add slash version if provided
        if let Some(version) = slash_version {
            content.push_str("path/to/unity/\n");
            content.push_str(&format!("{}\n", version));
            content.push_str("more/path/data\n");
        }
        
        // Add more bogus data
        for i in 0..bogus_data_size / 4 {
            content.push_str(&format!("error_message_{}\n", i));
            content.push_str("log_entry_data\n");
        }
        
        // Add underscore version if provided  
        if let Some(version) = underscore_version {
            content.push_str("version_info_block\n");
            content.push_str(&format!("{}\n", version));
            content.push_str("build_metadata\n");
        }
        
        // Add final bogus data
        for i in 0..bogus_data_size / 4 {
            content.push_str(&format!("final_symbol_{}\n", i));
            content.push_str("cleanup_data\n");
        }
        
        // Add parentheses version at the end if provided (should still be prioritized)
        if let Some(version) = parentheses_version {
            content.push_str("unity_build_info\n");
            content.push_str(&format!("{}\n", version));
            content.push_str("end_of_data\n");
        }
        
        content
    }

    #[test]
    fn prioritizes_parentheses_hash_over_other_formats_in_large_dataset() {
        let test_data = generate_test_data_with_versions(
            Some("2023.1.5f1 (abc123def456)"),
            Some("2022.3.2f1_xyz789uvw012"), 
            Some("2021.2.1f1/def456ghi789"),
            &["2020.1.0f1", "2019.4.2f1", "2024.1.0a1", "2018.3.5f1"],
            1000  // Large amount of bogus data
        );
        
        let result = Version::from_string_containing(&test_data);
        assert!(result.is_ok(), "Should extract version from large dataset");
        
        let version = result.unwrap();
        // Should prioritize the parentheses version even though it appears last
        assert_eq!(version.base.major, 2023);
        assert_eq!(version.base.minor, 1);
        assert_eq!(version.base.patch, 5);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 1);
    }

    #[test]
    fn prioritizes_underscore_hash_when_no_parentheses_version() {
        let test_data = generate_test_data_with_versions(
            None, // No parentheses version
            Some("2022.3.2f1_xyz789uvw012"), 
            Some("2021.2.1f1/def456ghi789"),
            &["2020.1.0f1", "2019.4.2f1", "2024.1.0a1"],
            800
        );
        
        let result = Version::from_string_containing(&test_data);
        assert!(result.is_ok(), "Should extract underscore hash version");
        
        let version = result.unwrap();
        // Should prioritize the underscore version over slash and standalone versions
        assert_eq!(version.base.major, 2022);
        assert_eq!(version.base.minor, 3);
        assert_eq!(version.base.patch, 2);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 1);
    }

    #[test]
    fn prioritizes_slash_hash_when_no_other_hash_formats() {
        let test_data = generate_test_data_with_versions(
            None, // No parentheses version
            None, // No underscore version
            Some("2021.2.1f1/def456ghi789"),
            &["2020.1.0f1", "2019.4.2f1", "2024.1.0a1", "2025.1.0b1"],
            600
        );
        
        let result = Version::from_string_containing(&test_data);
        assert!(result.is_ok(), "Should extract slash hash version");
        
        let version = result.unwrap();
        // Should prioritize the slash version over standalone versions
        assert_eq!(version.base.major, 2021);
        assert_eq!(version.base.minor, 2);
        assert_eq!(version.base.patch, 1);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 1);
    }

    #[test]
    fn falls_back_to_first_standalone_version_when_no_hash_versions() {
        let test_data = generate_test_data_with_versions(
            None, // No parentheses version
            None, // No underscore version  
            None, // No slash version
            &["2020.1.0f1", "2019.4.2f1", "2024.1.0a1", "2025.1.0b1"],
            400
        );
        
        let result = Version::from_string_containing(&test_data);
        assert!(result.is_ok(), "Should extract first standalone version");
        
        let version = result.unwrap();
        // Should find the first standalone version when no hash versions exist
        assert_eq!(version.base.major, 2020);
        assert_eq!(version.base.minor, 1);
        assert_eq!(version.base.patch, 0);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 1);
    }

    #[test]
    fn handles_multiple_versions_with_same_priority_returns_first_found() {
        let test_data = generate_test_data_with_versions(
            Some("2023.1.5f1 (abc123def456)"),
            None,
            None,
            &["2020.1.0f1"],
            200
        );
        
        // Add another parentheses version earlier in the data
        let mut modified_data = String::new();
        modified_data.push_str("Early data\n");
        modified_data.push_str("2024.2.1f1 (first123hash)\n");
        modified_data.push_str("More early data\n");
        modified_data.push_str(&test_data);
        
        let result = Version::from_string_containing(&modified_data);
        assert!(result.is_ok(), "Should extract first parentheses version");
        
        let version = result.unwrap();
        // Should find the first parentheses version encountered
        assert_eq!(version.base.major, 2024);
        assert_eq!(version.base.minor, 2);
        assert_eq!(version.base.patch, 1);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 1);
    }


    #[test]  
    fn handles_extremely_large_dataset_with_performance() {
        // Generate a very large dataset to test performance
        let test_data = generate_test_data_with_versions(
            Some("2023.3.10f1 (abc123def456)"),
            Some("2022.1.1f1_def456ghi789"),
            None,
            &[
                "2021.1.0f1", "2020.3.15f1", "2019.4.28f1", "2018.4.36f1",
                "2017.4.40f1", "2016.4.39f1", "2015.4.39f1", "5.6.7f1",
                "5.5.6f1", "5.4.6f1", "5.3.8f2", "5.2.5f1"
            ],
            5000  // Very large bogus dataset
        );
        
        let start = std::time::Instant::now();
        let result = Version::from_string_containing(&test_data);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Should handle large dataset");
        assert!(duration.as_millis() < 100, "Should parse large dataset quickly (took {:?})", duration);
        
        let version = result.unwrap();
        // Should still prioritize correctly even in large dataset
        assert_eq!(version.base.major, 2023);
        assert_eq!(version.base.minor, 3);
        assert_eq!(version.base.patch, 10);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 1);
    }

    #[test]
    fn prioritizes_versions_with_hashes() {
        // Test content with multiple versions, including ones with hashes
        let test_content = r#"
/lib64/ld-linux-x86-64.so.2
__libc_start_main
2020.2.0b2
2018.1.0b7
6000.2.0f1 (eed1c594c913)
6000.2.0f1_eed1c594c913
2022.2.0a1
2018.3.0a1
6000.2/respin/6000.2.0f1-517f89d850d1
5.0.0a1
6000.2.0f1.2588.6057
2017.2.0b1
6000.2.0f1
"#;

        let result = Version::from_string_containing(test_content);
        assert!(result.is_ok(), "Should successfully extract a version");

        let version = result.unwrap();
        // Should prioritize the version with parentheses hash: 6000.2.0f1 (eed1c594c913)
        assert_eq!(version.base.major, 6000);
        assert_eq!(version.base.minor, 2);
        assert_eq!(version.base.patch, 0);
        assert_eq!(version.release_type, ReleaseType::Final);
        assert_eq!(version.revision, 1);
    }

    #[test]
    fn handles_fallback_to_versions_without_hashes() {
        // Test content with only versions without hashes
        let test_content = r#"
Some random text
2020.2.0b2
More text
2018.1.0b7
Even more text
"#;

        let result = Version::from_string_containing(test_content);
        assert!(result.is_ok(), "Should successfully extract a version");

        let version = result.unwrap();
        // Should find the first valid version when no hashed versions exist
        assert_eq!(version.base.major, 2020);
        assert_eq!(version.base.minor, 2);
        assert_eq!(version.base.patch, 0);
        assert_eq!(version.release_type, ReleaseType::Beta);
        assert_eq!(version.revision, 2);
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
