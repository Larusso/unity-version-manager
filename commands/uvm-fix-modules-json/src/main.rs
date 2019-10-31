use console::style;
use log::{info, trace};
use std::fs::OpenOptions;
use std::io::{Result, Write};
use std::process;
use uvm_cli;
use uvm_core;
use uvm_core::unity::{Installations, Manifest};
use uvm_fix_modules_json::Options;

const USAGE: &str = "
uvm-fix-modules-json - Write the modules.json for the given unity version.

Usage:
  uvm-fix-modules-json [options] (--all | <version>...)
  uvm-fix-modules-json (-h | --help)

Options:
  --dry-run                         only prints modules to stdout
  --all                             generate modules.json for all installed editors
  -v, --verbose                     print more output
  -d, --debug                       print debug output
  --color WHEN                      Coloring: auto, always, never [default: auto]
  -h, --help                        show this help message and exit
";

fn generate_for_installation(options: Options) -> Result<()> {
    let installations = if options.all() {
        info!("generate modules.json for all installations");
        uvm_core::list_all_installations()
    } else {
        let v = options
            .version()
            .as_ref()
            .expect("expect a provided version");
        let installations: Installations = v
            .iter()
            .flat_map(|v| uvm_core::find_installation(v).into_iter())
            .collect();
        Ok(installations)
    }
    .unwrap();

    for i in installations {
        info!("{}", style(format!("generate modules.json for installation: {}", i.path().display())).yellow());
        let mut manifest = Manifest::load(i.version()).expect("a manifest");
        let c = i.installed_components();
        manifest.mark_installed_modules(c);
        let modules = manifest.into_modules();
        let j = serde_json::to_string_pretty(&modules).expect("export json");
        let output_path = i.path().join("modules.json");
        if options.dry_run() {
            eprintln!("modules json for {}", &i.version());
            eprintln!("output path: {}", output_path.display());
            println!("{}", j);
        } else {
            info!("{}", style(format!("write {}", output_path.display())).green());
            let output_path = i.path().join("modules.json");
            let mut f = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(output_path)?;
            write!(f, "{}", j)?;
            trace!("{}", j);
        }
    }
    Ok(())
}

fn main() {
    let options: Options = uvm_cli::get_options(USAGE).unwrap();
    trace!("generate modules.json");

    generate_for_installation(options).unwrap_or_else(|err| {
        let message = "Unable generate modules.json";
        eprintln!("{}", style(message).red());
        eprintln!("{}", style(err).red());
        process::exit(1);
    });

    eprintln!("{}", style("Done").green());
}
