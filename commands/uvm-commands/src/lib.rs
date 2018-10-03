#[macro_use]
extern crate serde_derive;
extern crate uvm_cli;
extern crate uvm_core;
extern crate console;

use console::Style;
use console::style;
use console::Term;
use std::fs;
use std::io;
use std::path::Path;
use uvm_cli::ColorOption;
use uvm_cli::Options;
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Deserialize)]
pub struct CommandsOptions {
    flag_verbose: bool,
    flag_path: bool,
    flag_list: bool,
    flag_1: bool,
    flag_color: ColorOption
}

impl CommandsOptions {
    pub fn path_only(&self) -> bool {
        self.flag_path
    }

    pub fn list(&self) -> bool {
        self.flag_list || self.single_column() || self.verbose()
    }

    pub fn single_column(&self) -> bool {
        self.flag_1
    }
}

impl uvm_cli::Options for CommandsOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
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

    pub fn exec(&self, options:CommandsOptions) -> io::Result<()>
    {
        let commands = uvm_cli::find_sub_commands()?;
        let out_style = Style::new().cyan();
        let path_style = Style::new().italic().green();

        let list = options.list();
        let path_only = options.path_only();
        let single_column = options.single_column();

        let seperator = match list || !self.stdout.is_term() { true => "\n", false => "  "};
        let output = commands.fold(String::new(), |out, command| {
            let mut new_line = out;

            if !path_only || (list && !single_column) {
                new_line += &format!("{}", out_style.apply_to(command.command_name().to_string()));
            }

            if list && !single_column {
                new_line += " - ";
            }

            if path_only || (list && !single_column) {
                new_line += &format!("{}", path_style.apply_to(command.path().display()));
            }
            new_line += seperator;
            new_line
        });
        self.stdout.write_line(&output)?;
        Ok(())
    }
}
