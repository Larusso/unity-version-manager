use crate::sys::version as version_impl;
use derive_more::Display;
use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::map_res,
    error::{context, convert_error, VerboseError},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::{cmp::Ordering, str::FromStr};

mod release_type;
mod revision_hash;
use crate::error::VersionError;
pub use release_type::ReleaseType;
pub use revision_hash::RevisionHash;

#[derive(Eq, Debug, Clone, Hash, PartialOrd, Display)]
#[display(fmt = "{}{}{}", base, release_type, revision)]
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
                let verbose_error = match err {
                    nom::Err::Error(e) | nom::Err::Failure(e) => e,
                    _ => VerboseError {
                        errors: vec![(s, nom::error::VerboseErrorKind::Context("unknown error"))],
                    },
                };
                Err(VersionError::ParsingFailed(convert_error(s, verbose_error)))
            }
        }
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
}

#[derive(Eq, Debug, Clone, Hash, Display)]
#[display(fmt = "{} ({})", version, revision)]
pub struct CompleteVersion{
    version: Version, 
    revision: RevisionHash
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


fn parse_release_type(input: &str) -> IResult<&str, ReleaseType, VerboseError<&str>> {
    context(
        "release type",
        map_res(alt((char('f'), char('b'), char('a'), char('p'))), |c| {
            ReleaseType::try_from(c)
        }),
    )(input)
}

fn parse_version(input: &str) -> IResult<&str, Version, VerboseError<&str>> {
    context(
        "version",
        tuple((
            context("major version", map_res(digit1, |s: &str| s.parse::<u64>())),
            char('.'),
            context("minor version", map_res(digit1, |s: &str| s.parse::<u64>())),
            char('.'),
            context("patch version", map_res(digit1, |s: &str| s.parse::<u64>())),
            context("release type", parse_release_type),
            context("revision", map_res(digit1, |s: &str| s.parse::<u64>())),
        )),
    )(input)
    .map(
        |(next_input, (major, _, minor, _, patch, release_type, revision))| {
            let base = semver::Version::new(major, minor, patch);
            (
                next_input,
                Version {
                    base,
                    release_type,
                    revision,
                },
            )
        },
    )
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

        assert!(version.base.major == 11, "parse correct major component");
        assert!(version.base.minor == 2, "parse correct minor component");
        assert!(version.base.patch == 3, "parse correct patch component");

        assert_eq!(version.release_type, ReleaseType::Final);
        assert!(version.revision == 4, "parse correct revision component");
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
