use std::path::PathBuf;
use std::fs::File;
use serde_derive::Deserialize;
use uvm_cli;
use uvm_cli::ColorOption;
use uvm_core::unity::Version;
use std::io::{self, Write};

#[derive(Debug, Deserialize)]
pub struct Options {
    arg_version: Vec<Version>,
    #[serde(default)]
    flag_output_dir: Option<PathBuf>,
    flag_name: String,
    flag_verbose: bool,
    flag_force: bool,
    flag_debug: bool,
    flag_color: ColorOption,
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

impl Options {
    pub fn version(&self) -> &Vec<Version> {
        &self.arg_version
    }

    pub fn name<V: AsRef<Version>>(&self, version:V) -> String {
        let name = &self.flag_name;
        let version = version.as_ref();
        name.as_str().replace("{version}", &version.to_string())
    }

    pub fn output<V: AsRef<Version>>(&self, version:V) -> io::Result<impl Write> {
        use std::fs::OpenOptions;
        if let Some(path) = &self.flag_output_dir {
            if !path.is_dir() {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "output path is not a directory"))
            } else {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .create_new(!self.flag_force)
                    .open(path.join(self.name(version)))
                    .and_then(|f| Ok(Output::File(f)))
            }
        } else {
            Ok(Output::default())
        }
    }

    pub fn output_path<V: AsRef<Version>>(&self, version:V) -> Option<PathBuf> {
        let path = self.flag_output_dir.as_ref()?;
        if !path.is_dir() {
            None
        } else {
            Some(path.join(self.name(version)))
        }
    }
}

impl uvm_cli::Options for Options {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn debug(&self) -> bool {
        self.flag_debug
    }

    fn color(&self) -> &ColorOption {
        &self.flag_color
    }
}
