use anyhow::{Context, Result};
use std::process;
use structopt::{
  clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli;

const COMMANDS: &str = "
COMMANDS:
  detect            Find which version of api was used to generate a project
  launch            Launch the current active version of api
  list              List api versions available
  install           Install specified api version
  install2          Install specified api version
  uninstall         Uninstall specified api version
  versions          List available Unity versions to install
  help              show command help and exit
";

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(),
            author = crate_authors!(),
            about = crate_description!(),
            setting = AppSettings::AllowExternalSubcommands,
            after_help = COMMANDS)]
struct Opts {
  #[structopt(subcommand)]
  sub: Subcommand,
}

#[derive(StructOpt, Debug, PartialEq)]
enum Subcommand {
  #[structopt(external_subcommand)]
  Command(Vec<String>),
}

impl Subcommand {
  fn exec(self) -> Result<i32> {
    let mut args = match self {
      Self::Command(args) => args,
    };
    let rest = args.split_off(1);

    let command = uvm_cli::sub_command_path(&args[0])?;
    uvm_cli::exec_command(command, rest).context("failed to execute subcommand")
  }
}

fn main() -> Result<()> {
  let opt = Opts::from_args();
  process::exit(opt.sub.exec()?);
}
