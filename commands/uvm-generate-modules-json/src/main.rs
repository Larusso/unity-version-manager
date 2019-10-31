use console::style;
use log::{info, trace};
use std::io::{Result, Write};
use std::process;
use uvm_cli;
use uvm_core;
use uvm_core::unity::Manifest;
use uvm_generate_modules_json::Options;

const USAGE: &str = "
uvm-generate-modules-json - Write the modules.json for the given unity version.

Usage:
  uvm-generate-modules-json [options] <version>...
  uvm-generate-modules-json (-h | --help)

Options:
  -o=PATH, --output-dir=PATH        the output path. default stdout
  -n=NAME, --name=NAME              name of the output file. [default: unity-{version}.json]
  -f, --force                       force override of existing files.
  -v, --verbose                     print more output
  -d, --debug                       print debug output
  --color WHEN                      Coloring: auto, always, never [default: auto]
  -h, --help                        show this help message and exit
";

fn generate_modules(options: Options) -> Result<()> {
    for version in options.version() {
        let mut output_handle = options.output(&version)?;
        let output_path = options.output_path(&version);

        let manifest = Manifest::load(version).unwrap();
        let modules = manifest.into_modules();
        let j = serde_json::to_string_pretty(&modules)?;

        if let Some(output_path) = output_path {
            info!("write modules to {}", output_path.display());
        }
        output_handle.write_all(j.as_bytes())?;
    }
    Ok(())
}

fn main() {
    let options: Options = uvm_cli::get_options(USAGE).unwrap();
    trace!("generate modules.json");

    generate_modules(options).unwrap_or_else(|err| {
        let message = "Unable generate modules.json";
        eprintln!("{}", style(message).red());
        eprintln!("{}", style(err).red());
        process::exit(1);
    });

    eprintln!("{}", style("Done").green());
}
