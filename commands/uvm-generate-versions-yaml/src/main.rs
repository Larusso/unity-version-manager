use std::{collections::{HashMap, BTreeMap}, str::FromStr};

use clap::Parser;
use cli::*;
use log::*;
use rayon::prelude::*;
use uvm_core::Version;
use uvm_live_platform::{ListVersions, UnityReleaseDownloadArchitecture, UnityReleaseStream};

mod cli;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    set_colors_enabled(&args.color);
    set_loglevel(args.debug.then(|| 2).unwrap_or(i32::from(args.verbose)));

    print!("{:?}", args);
    let streams = vec![
        UnityReleaseStream::Alpha,
        UnityReleaseStream::Beta,
        UnityReleaseStream::Lts,
        UnityReleaseStream::Tech,
    ];

    let versions = streams
        .par_iter()
        .map(|stream| {
            ListVersions::builder()
                .architecture(UnityReleaseDownloadArchitecture::X86_64)
                .autopage(true)
                .include_revision(true)
                .stream(stream.to_owned())
                .list()
        })
        .filter_map(|v| v.ok())
        .fold(
            || {
                let v: Vec<String> = vec![];
                v
            },
            |mut a, b| {
                let mut b_vec: Vec<String> = b.collect();
                a.append(&mut b_vec);
                a
            },
        )
        .flatten_iter()
        .filter_map(|v| Version::from_str(&v).ok())
        .map(|v| {
            let hash = v.version_hash().expect("expect revision hash to be included").to_owned();
            (v, hash)
        })
        .collect::<BTreeMap<Version, String>>();

    let s = serde_yaml::to_string(&versions)?;
    print!("{}", s);
    Ok(())
}
