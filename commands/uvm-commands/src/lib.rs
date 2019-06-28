#[macro_use]
extern crate serde_derive;

use uvm_cli;


use console::Style;
use console::Term;
use std::io;
use uvm_cli::ColorOption;
use uvm_cli::Options;

#[derive(Debug, Deserialize)]
pub struct CommandsOptions {
    flag_verbose: bool,
    flag_path: bool,
    flag_list: bool,
    flag_1: bool,
    flag_color: ColorOption,
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
}

impl Default for UvmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl UvmCommand {
    pub fn new() -> UvmCommand {
        UvmCommand {
            stdout: Term::stdout(),
        }
    }

    pub fn exec(&self, options: &CommandsOptions) -> io::Result<()> {
        let commands = uvm_cli::find_sub_commands()?;
        let out_style = Style::new().cyan();
        let path_style = Style::new().italic().green();

        let list = options.list();
        let path_only = options.path_only();
        let single_column = options.single_column();

        let seperator = if list || !self.stdout.is_term() {
            "\n"
        } else {
            "  "
        };
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
