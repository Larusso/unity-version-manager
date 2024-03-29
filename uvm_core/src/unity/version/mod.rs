use crate::unity::Installation;
use log::{debug, info};
use regex::Regex;
use semver;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::convert::{AsMut, AsRef, From, TryFrom};
use std::fmt;
use std::path::{Path, PathBuf};
use std::result;
use std::str::FromStr;
mod error;
mod hash;
pub use error::{Result, VersionError};
pub mod manifest;
pub mod module;

use crate::sys::unity::version as version_impl;

pub use self::hash::all_versions;
use self::hash::UnityHashError;

pub use self::version_impl::read_version_from_path;

#[derive(PartialEq, Eq, Ord, Hash, Debug, Clone, Copy, Deserialize)]
pub enum VersionType {
    Alpha,
    Beta,
    Patch,
    Final,
}

impl PartialOrd for VersionType {
    fn partial_cmp(&self, other: &VersionType) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, Debug, Clone, Hash, PartialOrd)]
pub struct Version {
    base: semver::Version,
    release_type: VersionType,
    revision: u64,
    hash: Option<String>,
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.base
            .cmp(&other.base)
            .then(self.release_type.cmp(&other.release_type))
            .then(self.revision.cmp(&other.revision))
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Version {
    pub fn new(
        major: u64,
        minor: u64,
        patch: u64,
        release_type: VersionType,
        revision: u64,
    ) -> Version {
        let base = semver::Version::new(major, minor, patch);
        Version {
            base,
            release_type,
            revision,
            hash: None,
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Version> {
        version_impl::read_version_from_path(path)
    }

    pub fn a(major: u64, minor: u64, patch: u64, revision: u64) -> Version {
        let base = semver::Version::new(major, minor, patch);
        Version {
            base,
            release_type: VersionType::Alpha,
            revision,
            hash: None,
        }
    }

    pub fn b(major: u64, minor: u64, patch: u64, revision: u64) -> Version {
        let base = semver::Version::new(major, minor, patch);
        Version {
            base,
            release_type: VersionType::Beta,
            revision,
            hash: None,
        }
    }

    pub fn p(major: u64, minor: u64, patch: u64, revision: u64) -> Version {
        let base = semver::Version::new(major, minor, patch);
        Version {
            base,
            release_type: VersionType::Patch,
            revision,
            hash: None,
        }
    }

    pub fn f(major: u64, minor: u64, patch: u64, revision: u64) -> Version {
        let base = semver::Version::new(major, minor, patch);
        Version {
            base,
            release_type: VersionType::Final,
            revision,
            hash: None,
        }
    }

    pub fn release_type(&self) -> &VersionType {
        &self.release_type
    }

    pub fn version_hash(&self) -> Result<String> {
        self.hash
            .as_ref()
            .map(|h| h.to_owned())
            .ok_or_else(|| VersionError::HashMissing {
                source: UnityHashError::Other,
                version: self.to_string(),
            })
            .or_else(|_err| {
                hash::hash_for_version(self).map_err(|source| VersionError::HashMissing {
                    source,
                    version: self.to_string(),
                })
            })
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

    #[cfg(unix)]
    pub fn find_version_in_file<P: AsRef<Path>>(path: P) -> Result<Version> {
        use std::process::{Command, Stdio};

        let path = path.as_ref();
        debug!("find unity version in Unity executable {}", path.display());

        let child = Command::new("strings")
            .arg("--")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output()?;

        if !output.status.success() {
            return Err(VersionError::ExecutableContainsNoVersion(
                path.display().to_string(),
            ));
        }

        let version = Version::from_str(&String::from_utf8_lossy(&output.stdout))?;
        debug!("found version {}", &version);
        Ok(version)
    }

    pub fn base(&self) -> &semver::Version {
        &self.base
    }

    pub fn as_semver(&self) -> semver::Version {
        let mut v = self.base.clone();
        if self.release_type != VersionType::Final {
            v.pre = semver::Prerelease::new(&format!("{}.{}", self.release_type, self.revision))
                .unwrap();
        }
        v
    }

    pub fn set_version_hash<S: AsRef<str>>(&mut self, hash: Option<S>) {
        self.hash = hash.map(|s| s.as_ref().to_owned());
    }

    pub fn has_version_hash(&self) -> bool {
        self.hash.is_some()
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        let eq = self.base == other.base && self.release_type == other.release_type && self.revision == other.revision;
        if self.hash.is_some() && other.hash.is_some() {
            return eq && self.hash == other.hash
        }
        eq
    }
}

impl From<(u64, u64, u64, u64)> for Version {
    fn from(tuple: (u64, u64, u64, u64)) -> Version {
        let (major, minor, patch, revision) = tuple;
        Version::f(major, minor, patch, revision)
    }
}

impl TryFrom<PathBuf> for Version {
    type Error = VersionError;

    fn try_from(path: PathBuf) -> Result<Self> {
        Version::from_path(path)
    }
}

impl TryFrom<&Path> for Version {
    type Error = VersionError;

    fn try_from(path: &Path) -> Result<Self> {
        Version::from_path(path)
    }
}

impl fmt::Display for VersionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            match *self {
                VersionType::Final => write!(f, "final"),
                VersionType::Patch => write!(f, "patch"),
                VersionType::Beta => write!(f, "beta"),
                VersionType::Alpha => write!(f, "alpha"),
            }
        } else {
            match *self {
                VersionType::Final => write!(f, "f"),
                VersionType::Patch => write!(f, "p"),
                VersionType::Beta => write!(f, "b"),
                VersionType::Alpha => write!(f, "a"),
            }
        }
    }
}

impl Default for VersionType {
    fn default() -> VersionType {
        VersionType::Final
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.base,
            self.release_type.to_string(),
            self.revision
        )
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

    fn from_str(s: &str) -> Result<Self> {
        let version_pattern =
            Regex::new(r"([0-9]{1,4})\.([0-9]{1,4})\.([0-9]{1,4})(f|p|b|a)([0-9]{1,4})( \(([a-z0-9]{12})\)|/([a-z0-9]{12}))?").unwrap();
        match version_pattern.captures(s) {
            Some(caps) => {
                let major: u64 = caps.get(1).map_or("0", |m| m.as_str()).parse().unwrap();
                let minor: u64 = caps.get(2).map_or("0", |m| m.as_str()).parse().unwrap();
                let patch: u64 = caps.get(3).map_or("0", |m| m.as_str()).parse().unwrap();

                let release_type = match caps.get(4).map_or("", |m| m.as_str()) {
                    "f" => Some(VersionType::Final),
                    "p" => Some(VersionType::Patch),
                    "b" => Some(VersionType::Beta),
                    "a" => Some(VersionType::Alpha),
                    _ => None,
                };

                let revision: u64 = caps.get(5).map_or("0", |m| m.as_str()).parse().unwrap();
                let hash = caps.get(7).or(caps.get(8)).map(|m| m.as_str().to_owned());
                let base = semver::Version::new(major, minor, patch);
                Ok(Version {
                    base,
                    revision,
                    release_type: release_type.unwrap(),
                    hash: hash,
                })
            }
            None => Err(VersionError::ParsingFailed(s.to_string())),
        }
    }
}

impl FromStr for VersionType {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "f" => Ok(VersionType::Final),
            "p" => Ok(VersionType::Patch),
            "b" => Ok(VersionType::Beta),
            "a" => Ok(VersionType::Alpha),
            "final" => Ok(VersionType::Final),
            "patch" => Ok(VersionType::Patch),
            "beta" => Ok(VersionType::Beta),
            "alpha" => Ok(VersionType::Alpha),
            _ => Err(VersionError::VersionTypeParsingFailed(s.to_string())),
        }
    }
}

impl From<Installation> for Version {
    fn from(item: Installation) -> Self {
        item.version_owned()
    }
}

pub fn fetch_matching_version<I: Iterator<Item = Version>>(
    versions: I,
    version_req: semver::VersionReq,
    release_type: VersionType,
) -> Result<Version> {
    versions
        .filter(|version| {
            let semver_version = if version.release_type() < &release_type {
                debug!(
                    "version {} release type is smaller than specified type {:#}",
                    version, release_type
                );
                let mut semver_version = version.base().clone();
                semver_version.pre = semver::Prerelease::new(&format!(
                    "{}.{}",
                    version.release_type, version.revision
                ))
                .unwrap();
                semver_version
            } else {
                let b = version.base().clone();
                debug!(
                    "use base semver version {} of {} for comparison",
                    b, version
                );
                b
            };

            let is_match = version_req.matches(&semver_version);
            if is_match {
                info!("version {} is a match", version);
            } else {
                info!("version {} is not a match", version);
            }

            is_match
        })
        .max()
        .ok_or_else(|| VersionError::NoMatch(version_req.to_string()))
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
        when_version_has_only_zero_digits: "0.0.0f0",
        when_version_has_optional_hash_project_settings_style: "2020.3.38f1 (8f5fde82e2dc)",
        when_version_has_optional_hash_unity_hub_style: "2020.3.38f1/8f5fde82e2dc"
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

        assert!(version.base.major == 1, "parse correct major component");
        assert!(version.base.minor == 2, "parse correct minor component");
        assert!(version.base.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, VersionType::Final);
        assert!(version.revision == 4, "parse correct revision component");
        assert!(version.hash.is_none(), "parse correct optional hash")
    }

    #[test]
    fn splits_version_string_into_components_with_hash() {
        let version_string = "1.2.3f4 (abcdefghijkm)";
        let version = Version::from_str(version_string).ok().unwrap();

        assert!(version.base.major == 1, "parse correct major component");
        assert!(version.base.minor == 2, "parse correct minor component");
        assert!(version.base.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, VersionType::Final);
        assert!(version.revision == 4, "parse correct revision component");
        assert!(version.hash.unwrap() == "abcdefghijkm", "parse correct optional hash")
    }

    #[test]
    fn splits_version_string_into_components_with_hash_unity_hub_style() {
        let version_string = "1.2.3f4/abcdefghijkm";
        let version = Version::from_str(version_string).ok().unwrap();

        assert!(version.base.major == 1, "parse correct major component");
        assert!(version.base.minor == 2, "parse correct minor component");
        assert!(version.base.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, VersionType::Final);
        assert!(version.revision == 4, "parse correct revision component");
        assert!(version.hash.unwrap() == "abcdefghijkm", "parse correct optional hash")
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

    #[test]
    fn fetch_hash_for_known_version() {
        let version = Version::f(2017, 1, 0, 2);
        assert_eq!(
            version.version_hash().unwrap(),
            String::from("66e9e4bfc850")
        );
    }

    #[test]
    fn compares_versions() {
        let version_a = Version::from_str("1.2.3f4").ok().unwrap();
        let version_b = Version::from_str("1.2.3f4").ok().unwrap();
        assert_eq!(version_a, version_b, "testing version equality");

        let version_c = Version::from_str("1.2.3f4").ok().unwrap();
        let version_d = Version::from_str("1.2.3f5").ok().unwrap();
        assert_ne!(version_c, version_d, "testing version nonequality"); 

        let version_c = Version::from_str("1.2.3f4").ok().unwrap();
        let version_d = Version::from_str("1.2.3f4/1234567890ab").ok().unwrap();
        assert_eq!(version_c, version_d, "testing version equality when one version has hash other not"); 

        let version_c = Version::from_str("1.2.3f4/0987654321ab").ok().unwrap();
        let version_d = Version::from_str("1.2.3f4/1234567890ab").ok().unwrap();
        assert_ne!(version_c, version_d, "testing version equality when one version hash is different"); 
    }

    #[cfg(unix)]
    #[test]
    fn reads_version_from_binary_file() {
        use std::io::Write;
        use tempfile::Builder;

        let mut test_file = Builder::new()
            .prefix("version_binary")
            .rand_bytes(5)
            .tempfile()
            .unwrap();

        let version = "2018.2.1f2";
        let version_hash = "dft74dsds844";

        //Some known result patterns
        let test_value_1 = format!("Unity {}\n", version);
        let test_value_2 = format!("{}_{}\n", version, version_hash);
        let test_value_3 = format!("{} ({})\n", version, version_hash);
        let test_value_4 = format!("Mozilla/5.0 (MacIntel; ) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/37.0.2062.94 Safari/537.36 Unity/{} (unity3d.com;\n", version);
        let test_value_5 = format!("Invalid serialized file version. File: \"%s\". Expected version: {}. Actual version: %s.\n", version);
        let test_value_6 = format!(
            "UnityPlayer/{} (UnityWebRequest/1.0, libcurl/7.52.0-DEV)\n",
            version
        );

        let f = test_file.as_file_mut();
        let random_bytes: Vec<u8> = (0..2048).map(|_| rand::random::<u8>()).collect();

        f.write_all(&random_bytes).unwrap();
        f.write_all(test_value_1.as_bytes()).unwrap();
        f.write_all(&random_bytes).unwrap();
        f.write_all(test_value_2.as_bytes()).unwrap();
        f.write_all(&random_bytes).unwrap();
        f.write_all(test_value_3.as_bytes()).unwrap();
        f.write_all(&random_bytes).unwrap();
        f.write_all(test_value_4.as_bytes()).unwrap();
        f.write_all(&random_bytes).unwrap();
        f.write_all(test_value_5.as_bytes()).unwrap();
        f.write_all(&random_bytes).unwrap();
        f.write_all(test_value_6.as_bytes()).unwrap();
        f.write_all(&random_bytes).unwrap();

        let v = Version::find_version_in_file(test_file.path()).unwrap();
        assert_eq!(v, Version::f(2018, 2, 1, 2));
    }

    #[cfg(unix)]
    #[test]
    fn fails_to_read_version_from_binary_file_if_verion_can_not_be_found() {
        use std::io::Write;
        use tempfile::Builder;

        let mut test_file = Builder::new()
            .prefix("version_binary")
            .rand_bytes(5)
            .tempfile()
            .unwrap();

        let f = test_file.as_file_mut();
        let random_bytes: Vec<u8> = (0..8000).map(|_| rand::random::<u8>()).collect();

        f.write_all(&random_bytes).unwrap();
        let v = Version::find_version_in_file(test_file.path());
        assert!(v.is_err());
    }

    #[test]
    fn fetch_hash_for_unknown_version_yields_none() {
        let version = Version::f(2080, 2, 0, 2);
        assert!(version.version_hash().is_err());
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {
            let _ = Version::from_str(s);
        }

        #[test]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            Version::from_str(s).ok().unwrap();
        }

        #[test]
        fn parses_version_back_to_original(major in 0u64..9999, minor in 0u64..9999, patch in 0u64..9999, revision in 0u64..9999 ) {
            let v1 = Version {
                base: semver::Version::new(major, minor, patch),
                revision,
                release_type: VersionType::Final,
                hash: None
            };

            let v2 = Version::from_str(&format!("{:04}.{:04}.{:04}f{:04}", major, minor, patch, revision)).ok().unwrap();
            prop_assert_eq!(v1, v2);
        }

        #[test]
        fn create_version_from_tuple(major in 0u64..9999, minor in 0u64..9999, patch in 0u64..9999, revision in 0u64..9999 ) {
            let v1 = Version {
                base: semver::Version::new(major, minor, patch),
                revision,
                release_type: VersionType::Final,
                hash: None
            };

            let v2:Version = (major, minor, patch, revision).into();
            prop_assert_eq!(v1, v2);
        }

        #[test]
        fn create_version_final_versions(major in 0u64..9999, minor in 0u64..9999, patch in 0u64..9999, revision in 0u64..9999 ) {
            let v1 = Version {
                base: semver::Version::new(major, minor, patch),
                revision,
                release_type: VersionType::Final,
                hash: None
            };

            let v2:Version = Version::f(major, minor, patch, revision);
            prop_assert_eq!(v1, v2);
        }

        #[test]
        fn create_version_beta_versions(major in 0u64..9999, minor in 0u64..9999, patch in 0u64..9999, revision in 0u64..9999 ) {
            let v1 = Version {
                base: semver::Version::new(major, minor, patch),
                revision,
                release_type: VersionType::Beta,
                hash: None
            };

            let v2:Version = Version::b(major, minor, patch, revision);
            prop_assert_eq!(v1, v2);
        }

        #[test]
        fn create_version_alpha_versions(major in 0u64..9999, minor in 0u64..9999, patch in 0u64..9999, revision in 0u64..9999 ) {
            let v1 = Version {
                base: semver::Version::new(major, minor, patch),
                revision,
                release_type: VersionType::Alpha,
                hash: None
            };

            let v2:Version = Version::a(major, minor, patch, revision);
            prop_assert_eq!(v1, v2);
        }

        #[test]
        fn create_version_patch_versions(major in 0u64..9999, minor in 0u64..9999, patch in 0u64..9999, revision in 0u64..9999 ) {
            let v1 = Version {
                base: semver::Version::new(major, minor, patch),
                revision,
                release_type: VersionType::Patch,
                hash: None
            };

            let v2:Version = Version::p(major, minor, patch, revision);
            prop_assert_eq!(v1, v2);
        }
    }
}
