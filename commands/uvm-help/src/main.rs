use anyhow::{Context, Result};
use structopt::{clap::crate_authors, clap::crate_description, clap::crate_version, StructOpt};
use uvm_cli;

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    /// Command name to print help text for
    command: String,
}

fn main() -> Result<()> {
    let opt = Opts::from_args();
    let command = uvm_cli::sub_command_path(&opt.command)
        .context(format!("failed to lookup command {}", opt.command))?;
    uvm_cli::exec_command(command, vec!["--help"])?;
    Ok(())
}
