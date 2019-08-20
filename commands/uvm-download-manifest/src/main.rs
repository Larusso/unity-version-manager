use log::{error, info, debug};
use std::io;
use std::io::prelude::*;
use uvm_core::platform::Platform;
use uvm_core::unity::urls::IniUrlBuilder;
use uvm_core::Version;
use uvm_download_manifest::Options;
use console::style;

const USAGE: &str = "
uvm-download-manifest - Download the manifest.ini file for a given unity version.

Usage:
  uvm-download-manifest [options] <version>
  uvm-download-manifest (-h | --help)

Options:
  -o=PATH, --output-dir=PATH        the output path. default stdout
  -n=NAME, --name=NAME              name of the output file. [default: unity-{version}-{platform}.ini]
  -p=PLATFORM, --platform=PLATFORM  the platform to download (macos,win,linux). defaults to current platform.
  -f, --force                       force override of existing files.
  -v, --verbose                     print more output
  -d, --debug                       print debug output
  --color WHEN                      Coloring: auto, always, never [default: auto]
  -h, --help                        show this help message and exit
";

fn main() {
    run().unwrap_or_else(|err| {
        error!("failed to download manifest");
        error!("{}", err);
    })
}

fn download_manifest<V, W>(version: V, platform: Platform, output: W) -> io::Result<()>
where
    V: AsRef<Version>,
    W: Write,
{
    let url = IniUrlBuilder::new().platform(platform).build(version)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    info!("manifest URL: {}", url.as_str());
    let response = reqwest::get(url.into_url())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    let mut buf_reader = io::BufReader::new(response);
    let mut buf_writer = io::BufWriter::new(output);
    let mut bytes:Vec<u8> = Vec::new();
    buf_reader.read_to_end(&mut bytes)?;
    buf_writer.write_all(&bytes)?;

    eprintln!(
        "{}",
        style("Download complete").cyan(),
    );
    Ok(())
}

fn run() -> io::Result<()> {
    let options: Options = uvm_cli::get_options(USAGE).unwrap();
    debug!("{:?}", options);
    info!(
        "download manifest for version {} and platform {}",
        options.version(),
        options.platform()
    );
    let output_handle = options.output()?;
    if let Some(output_path) = options.output_path() {
        info!("write manifest to {}", output_path.display());
    }
    download_manifest(options.version(), options.platform(), output_handle)
}
