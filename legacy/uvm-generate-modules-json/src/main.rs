use anyhow::Result;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use uvm_core::Version;
use console::style;
use log::{info, trace};
use std::io;
use uvm_cli;
use uvm_core;
use uvm_core::unity::Manifest;
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};
use structopt::{clap::AppSettings, clap::crate_authors, clap::crate_description, clap::crate_version, StructOpt};

const SETTINGS: &'static [AppSettings] = &[AppSettings::ColoredHelp, AppSettings::DontCollapseArgsInUsage];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    #[structopt(name = "version", min_values(1), required(true))]
    version: Vec<Version>,

    /// the output path. default stdout
    #[structopt(short, long)]
    output_dir: Option<PathBuf>,

    /// name of the output file.
    #[structopt(short, long, default_value = "api-{version}.json")]
    name: String,

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
    pub fn output<V: AsRef<Version>>(&self, version:V) -> io::Result<impl Write> {
        use std::fs::OpenOptions;
        if let Some(path) = &self.output_dir {
            if !path.is_dir() {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "output path is not a directory"))
            } else {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .create_new(!self.force)
                    .open(path.join(self.name(version)))
                    .and_then(|f| Ok(Output::File(f)))
            }
        } else {
            Ok(Output::default())
        }
    }

    pub fn name<V: AsRef<Version>>(&self, version:V) -> String {
        let name = &self.name;
        let version = version.as_ref();
        name.as_str().replace("{version}", &version.to_string())
    }

    pub fn output_path<V: AsRef<Version>>(&self, version:V) -> Option<PathBuf> {
        let path = self.output_dir.as_ref()?;
        if !path.is_dir() {
            None
        } else {
            Some(path.join(self.name(version)))
        }
    }
}

fn main() -> Result<()> {
    let opt = Opts::from_args();
    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));
    trace!("generate modules.json");

    for version in &opt.version {
        let mut output_handle = opt.output(&version)?;
        let output_path = opt.output_path(&version);

        let manifest = Manifest::load(&version).map_err(|_| {
            io::Error::new(io::ErrorKind::NotFound, "Unable to load manifest")
        })?;

        if let Some(output_path) = output_path {
            info!("write modules to {}", output_path.display());
        }
        manifest.write_modules_json(&mut output_handle)?;
    }
    eprintln!("");
    eprintln!("{}", style("Done").green());
    Ok(())
}
