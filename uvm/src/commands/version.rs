use clap::Args;
use clap::Subcommand;
use console::style;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::{debug, info};
use semver::VersionReq;
use std::io;
use std::str::FromStr;
use std::time::Duration;
use unity_version::Version;
use uvm_live_platform::UnityReleaseDownloadArchitecture;
use uvm_live_platform::UnityReleaseDownloadPlatform;
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

        /// List all matching versions
        #[arg(short, long)]
        all: bool,
    },
    Latest {
        #[command(flatten)]
        filter: VersionFilter,
    },
}

#[derive(Args, Debug)]
struct VersionFilter {
    #[arg(short, long = "stream", value_enum)]
    streams: Vec<UnityReleaseStream>,

    #[arg(short, long = "entitlement", value_enum)]
    entitlements: Vec<UnityReleaseEntitlement>,

    #[arg(long = "architecture", value_enum)]
    architectures: Vec<UnityReleaseDownloadArchitecture>,

    #[arg(long = "platform", value_enum)]
    platforms: Vec<UnityReleaseDownloadPlatform>,

    #[arg(long)]
    /// Refresh the cache
    refresh: bool,

    #[arg(long)]
    /// Disable the cache
    no_cache: bool,
}

impl VersionCommand {
    pub fn execute(self) -> io::Result<i32> {
        let command = self.command;

        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                // For more spinners check out the cli-spinners project:
                // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
        );
        pb.set_message(format!("{}", style("fetching versions").yellow()));

        match command {
            Command::Latest { filter } => {
                let versions_builder = uvm_live_platform::ListVersions::builder()
                    .with_architectures(filter.architectures)
                    .with_platforms(filter.platforms)
                    .autopage(false)
                    .with_streams(filter.streams)
                    .with_entitlements(filter.entitlements)
                    .with_refresh(filter.refresh)
                    .without_cache(filter.no_cache)
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
                pb.finish_with_message("latest version");
                println!("{}", style(version).green().bold());
            }
            Command::Matching {
                version_req,
                filter,
                all,
            } => {
                let versions_builder = uvm_live_platform::ListVersions::builder()
                    .with_architectures(filter.architectures)
                    .with_platforms(filter.platforms)
                    .autopage(true)
                    .with_streams(filter.streams)
                    .with_entitlements(filter.entitlements);

                let versions = versions_builder
                    .list()
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{}", err)))?;

                let versions = versions
                    .into_iter()
                    .filter_map(|v| Version::from_str(&v).ok());

                debug!("versions: {:#?}", versions);

                if all {
                    let versions = Self::fetch_matching_versions(versions, version_req);
                    pb.finish_with_message("all matching versions");
                    info!("all matching versions:");
                    for version in versions {
                        println!("{}", style(version).green().bold());
                    }
                } else {
                    let version = Self::fetch_matching_version(versions, version_req)?;
                    pb.finish_with_message("highest matching version");
                    println!("{}", style(version).green().bold());
                }
            }
        }

        Ok(0)
    }

    fn fetch_matching_versions<I: Iterator<Item = Version>>(
        versions: I,
        version_req: VersionReq,
    ) -> impl Iterator<Item = Version> {
        versions.filter(move |version| {
            let b = version.base().clone();
            debug!(
                "use base semver version {} of {} for comparison",
                b, version
            );
            let semver_version = b;

            let is_match = &version_req.matches(&semver_version);
            if *is_match {
                info!("version {} is a match", version);
                true
            } else {
                info!("version {} is not a match", version);
                false
            }
        })
    }

    fn fetch_matching_version<I: Iterator<Item = Version>>(
        versions: I,
        version_req: VersionReq,
    ) -> io::Result<Version> {
        Self::fetch_matching_versions(versions, version_req.clone())
            .max()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "No version found matching reg: {}",
                        &version_req.to_string()
                    ),
                )
            })
    }
}
