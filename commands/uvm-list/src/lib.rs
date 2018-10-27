#[macro_use]
extern crate serde_derive;
extern crate uvm_cli;
extern crate uvm_core;
extern crate console;
extern crate log;

use log::info;
use log::trace;
use console::Style;
use console::Term;
use std::io;
use uvm_cli::ColorOption;
use uvm_cli::Options;

#[derive(Debug, Deserialize)]
pub struct ListOptions {
    flag_hub: bool,
    flag_verbose: bool,
    flag_debug: bool,
    flag_path: bool,
    flag_color: ColorOption
}

impl ListOptions {
    pub fn path_only(&self) -> bool {
        self.flag_path
    }

    pub fn use_hub(&self) -> bool {
        self.flag_hub
    }
}

impl uvm_cli::Options for ListOptions {
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

pub struct UvmCommand {
    stdout: Term,
    stderr: Term
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
            stderr: Term::stderr(),
        }
    }

    pub fn exec(&self, options:ListOptions) -> io::Result<()>
    {
        let current_version = uvm_core::current_installation().ok();
        let list_function = if options.use_hub() {
            info!("fetch installations from unity hub");
            uvm_core::unity::hub::list_installations
        } else {
            info!("fetch installations from uvm");
            uvm_core::list_installations
        };

        if let Ok(installations) = list_function() {
            self.stderr.write_line("Installed Unity versions:")?;
            let verbose = options.verbose();
            let path_only = options.path_only();

            let output = installations.fold(String::new(), |out, installation| {
                let mut out_style = Style::new().cyan();
                let mut path_style = Style::new().italic().green();

                if let &Some(ref current) = &current_version {
                    if current == &installation {
                        out_style = out_style.yellow().bold();
                        path_style = path_style.italic().yellow();
                    }
                }
                let mut new_line = out;

                if path_only == false {
                    new_line += &format!("{}", out_style.apply_to(installation.version().to_string()));
                }

                if verbose {
                    new_line += " - ";
                }

                if verbose || path_only {
                    new_line += &format!("{}", path_style.apply_to(installation.path().display()));
                }
                new_line += "\n";
                new_line
            });

            self.stdout.write_line(&output)?;
        };

        Ok(())
    }
}
