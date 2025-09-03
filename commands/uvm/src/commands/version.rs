use clap::Subcommand;
use clap::Args;
use log::{debug, info};
use semver::VersionReq;
use std::io;
use std::str::FromStr;
use console::style;
use unity_version::Version;
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

        #[command(flatten)]
        filter: VersionFilter,

    },
    Latest {
        #[command(flatten)]
        filter: VersionFilter,
    }
}

#[derive(Args, Debug)]
struct VersionFilter {
    #[arg(short, long="stream", value_enum)]
    streams: Vec<UnityReleaseStream>,

    #[arg(short, long="entitlement", value_enum)]
    entitlements: Vec<UnityReleaseEntitlement>,
}


impl VersionCommand {
    pub fn execute(self) -> io::Result<i32> {
        let command = self.command;
        
        match command {
            Command::Latest { filter} => {
                let versions_builder = uvm_live_platform::ListVersions::builder()
                    .for_current_system()
                    .autopage(false)
                    .with_streams(filter.streams)
                    .with_entitlements(filter.entitlements)
                    .limit(1);
                let versions = versions_builder
                    .list()
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{}", err)))?;
                let version = versions.into_iter().nth(0).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("No version found matching for provided filter"),
                    )
                })?;
                info!("highest matching version:");
                eprintln!("{}", style(version).green().bold()); 
            },
            Command::Matching { version_req, filter} => {
                let versions_builder = uvm_live_platform::ListVersions::builder()
                    .for_current_system()
                    .autopage(true)
                    .with_streams(filter.streams)
                    .with_entitlements(filter.entitlements);

                let versions = versions_builder
                    .list()
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{}", err)))?;

                let versions = versions.into_iter().filter_map(|v| Version::from_str(&v).ok());

                debug!("versions: {:#?}", versions);

                let version = Self::fetch_matching_version(versions, version_req)?;
                info!("highest matching version:");
                eprintln!("{}", style(version).green().bold()); 
            }
        }

        Ok(0)
    }

    fn fetch_matching_version<I: Iterator<Item = Version>>(
        versions: I,
        version_req: VersionReq,
    ) -> io::Result<Version> {
        versions
            .filter(|version| {
                // let semver_version = if version.release_type() < release_type {
                //     debug!(
                //         "version {} release type is smaller than specified type {:#}",
                //         version, release_type
                //     );
                //     let mut semver_version = version.base().clone();
                //     semver_version.pre = semver::Prerelease::new(&format!(
                //         "{}.{}",
                //         version.release_type(),
                //         version.revision()
                //     ))
                //     .unwrap();
                //     semver_version
                // } else {
                    let b = version.base().clone();
                    debug!(
                        "use base semver version {} of {} for comparison",
                        b, version
                    );
                    let semver_version = b;
                // };

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

