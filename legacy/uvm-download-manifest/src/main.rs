use anyhow::Result;
use log::{info};
use std::io;
use std::fs::File;
use std::io::Write;
use std::io::prelude::*;
use uvm_core::unity::urls::IniUrlBuilder;
use uvm_core::Version;
use console::style;
use std::path::PathBuf;
use uvm_core::platform::Platform;
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};
use structopt::{clap::AppSettings, clap::crate_authors, clap::crate_description, clap::crate_version, StructOpt};

const SETTINGS: &'static [AppSettings] = &[AppSettings::ColoredHelp, AppSettings::DontCollapseArgsInUsage];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    /// The api version to download the manifest.ini file for in the form of `2018.1.0f3`
    version: Version,

    /// the output path. default stdout
    #[structopt(short, long)]
    output_dir: Option<PathBuf>,

    /// name of the output file.
    #[structopt(short, long, default_value = "api-{version}-{platform}.ini")]
    name: String,

    /// the platform to download (macos,win,linux).
    #[structopt(short, long)]
    platform: Option<Platform>,

    /// force override of existing files. 
    #[structopt(short, long)]
    force: bool,
    
    /// print debug output
    #[structopt(short, long)]
    debug: bool,

    /// print more output
    #[structopt(short, long, parse(from_occurrences))]
    verbose: i32,

    /// Color:.
    #[structopt(short, long, possible_values = &ColorOption::variants(), case_insensitive = true, default_value)]
    color: ColorOption,
}

pub enum Output {
    File(File),
    Stdout,
}

impl Default for Output {
    fn default() -> Self {
        Output::Stdout
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use Output::*;
        match self {
            File(x) => x.write(buf),
            _ => console::Term::stdout().write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        use Output::*;
        match self {
            File(x) => x.flush(),
            _ => console::Term::stdout().flush(),
        }
    }
}

impl Opts {
    pub fn output(&self) -> io::Result<impl Write> {
        use std::fs::OpenOptions;
        if let Some(path) = &self.output_dir {
            if !path.is_dir() {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "output path is not a directory"))
            } else {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .create_new(!self.force)
                    .open(path.join(self.name()))
                    .and_then(|f| Ok(Output::File(f)))
            }

        } else {
            Ok(Output::default())
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform.unwrap_or_default()
    }

    pub fn name(&self) -> String {
        let name = &self.name;
        let name = name.as_str().replace("{version}", &self.version.to_string());
        let name = name.as_str().replace("{platform}", &self.platform().to_string());
        name
    }

    pub fn output_path(&self) -> Option<PathBuf> {
        let path = self.output_dir.as_ref()?;
        if !path.is_dir() {
            None
        } else {
            Some(path.join(self.name()))
        }
    }
}

fn main() -> Result<()> {
    let opt = Opts::from_args();
  
    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));
   
    info!(
        "download manifest for version {} and platform {}",
        opt.version,
        opt.platform()
    );

    let output_handle = opt.output()?;
    if let Some(output_path) = opt.output_path() {
        info!("write manifest to {}", output_path.display());
    }
    download_manifest(&opt.version, opt.platform(), output_handle)?;

    Ok(())
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
