use serde_derive::Deserialize;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use uvm_cli;
use uvm_cli::ColorOption;
use uvm_core::unity::Version;
use uvm_core::platform::Platform;

#[derive(Debug, Deserialize)]
pub struct Options {
    arg_version: Version,

    #[serde(default)]
    flag_platform: Option<Platform>,
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
    pub fn version(&self) -> &Version {
        &self.arg_version
    }

    pub fn platform(&self) -> Platform {
        self.flag_platform.unwrap_or_default()
    }

    pub fn name(&self) -> String {
        let name = &self.flag_name;
        let name = name.as_str().replace("{version}", &self.version().to_string());
        let name = name.as_str().replace("{platform}", &self.platform().to_string());
        name
    }

    pub fn output(&self) -> io::Result<impl Write> {
        use std::fs::OpenOptions;
        if let Some(path) = &self.flag_output_dir {
            if !path.is_dir() {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "output path is not a directory"))
            } else {
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .create_new(!self.flag_force)
                    .open(path.join(self.name()))
                    .and_then(|f| Ok(Output::File(f)))
            }

        } else {
            Ok(Output::default())
        }
    }

    pub fn output_path(&self) -> Option<PathBuf> {
        let path = self.flag_output_dir.as_ref()?;
        if !path.is_dir() {
            None
        } else {
            Some(path.join(self.name()))
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
