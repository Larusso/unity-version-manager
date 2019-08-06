use uvm_cli;
use uvm_core;
use log::{trace,debug,info};
use uvm_generate_modules_json::Options;
use uvm_core::unity::{Installations, Manifest, Modules};

const USAGE: &str = "
uvm-detect - Write the modules.json for the given unity version.

Usage:
  uvm-generate-mopdules-json [options] (--all | <version>)
  uvm-generate-mopdules-json (-h | --help)

Options:
  --all             generate modules.json for all installed editors
  -v, --verbose     print more output
  -d, --debug       print debug output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
";

fn main() {
    let options:Options = uvm_cli::get_options(USAGE).unwrap();
    trace!("generate modules.json");

    let installations = if options.all() {
        info!("generate modules.json for all installations");
        uvm_core::list_all_installations()
    } else {
        let v = options.version().as_ref().expect("expect a provided version");
        let installations:Installations = uvm_core::find_installation(v).into_iter().collect();
        Ok(installations)
    }.unwrap();

    for i in installations {
        debug!("installation: {}", i);
        let manifest = Manifest::load(i.version()).expect("a manifest");
        let modules:Modules = manifest.into();

        let j = serde_json::to_string_pretty(&modules).expect("export json");
        println!("{}", j);
    }

    eprintln!("Done");
}
