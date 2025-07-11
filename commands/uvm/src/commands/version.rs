use clap::Subcommand;
use clap::Args;
use log::{debug, info};
use semver::VersionReq;
use std::io;
use std::str::FromStr;
use console::style;
use unity_version::{ReleaseType, Version};
use uvm_live_platform::{UnityReleaseEntitlement, UnityReleaseStream};

#[derive(Args, Debug)]
pub struct VersionCommand {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Return version matching version req.
    Matching {
        /// The version requirement string
        ///
        /// The version requirement string will be converted to `semver::VersionReq`
        /// See https://docs.rs/semver/1.0.2/semver/struct.VersionReq.html for usage.
        version_req: VersionReq,

        /// The api release type
        ///
        /// The release type to limit the search for.
        #[arg(value_enum, default_value = "final")]
        release_type: ReleaseType,

        #[arg(short, long="stream", value_enum)]
        streams: Vec<UnityReleaseStream>,

        #[arg(short, long="entitlement", value_enum)]
        entitlements: Vec<UnityReleaseEntitlement>,

    },
    Latest {
        #[arg(value_enum, default_value = "final")]
        release_type: ReleaseType,
    },
}


impl VersionCommand {
    pub fn execute(self) -> io::Result<i32> {
        let command = self.command;
        let (version_req, version_type, release_streams, release_entitlements) = match command {
            Command::Latest { release_type } => (
                VersionReq::parse("*").expect("valid VersionReq"),
                release_type,
                vec![],
                vec![]
            ),
            Command::Matching {
                version_req,
                release_type,
                streams,
                entitlements,
            } => (version_req, release_type, streams, entitlements),
        };

        let mut versions_builder = uvm_live_platform::ListVersions::builder()
            .for_current_system()
            .autopage(true)
            .with_streams(release_streams)
            .with_entitlements(release_entitlements);

        let versions = versions_builder
            .list()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{}", err)))?;

        let versions = versions.into_iter().filter_map(|v| Version::from_str(&v).ok());

        let version = Self::fetch_matching_version(versions, version_req, version_type)?;
        info!("highest matching version:");
        eprintln!("{}", style(version).green().bold());

        Ok(0)
    }

    fn fetch_matching_version<I: Iterator<Item = Version>>(
        versions: I,
        version_req: VersionReq,
        release_type: ReleaseType,
    ) -> io::Result<Version> {
        versions
            .filter(|version| {
                let semver_version = if version.release_type() < release_type {
                    debug!(
                        "version {} release type is smaller than specified type {:#}",
                        version, release_type
                    );
                    let mut semver_version = version.base().clone();
                    semver_version.pre = semver::Prerelease::new(&format!(
                        "{}.{}",
                        version.release_type(),
                        version.revision()
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
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No version found matching reg: {}", version_req.to_string()),
                )
            })
    }
}

