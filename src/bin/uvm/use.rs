extern crate console;
extern crate uvm;

use std::path::Path;
const USAGE: &'static str = "
uvm-use - Use specific version of unity.

Usage:
  uvm-use [options] <version>
  uvm-use (-h | --help)

Options:
  -v, --verbose     print more output
  -h, --help        show this help message and exit
";

fn main() {
    let o = uvm::cli::get_use_options(USAGE).unwrap();

    if let Ok(current_installtion) = uvm::current_installation() {
        if current_installtion.version() == &o.version {
            println!("Version {} already active", &o.version);
            return ();
        }
    }

    if let Ok(mut installations) = uvm::list_installations() {
        if let Some(installation) = installations.find(|i| i.version() == &o.version) {
            println!("Found installation {:?}", installation);
            //cleanup / check symlink
            //set new symlink
            let active_path = Path::new("/Applications/Unity");
            if active_path.exists() {
                if std::fs::remove_file(active_path).is_err() {
                    println!("Can't unlink current version");
                    return ();
                }
            }
            if std::os::unix::fs::symlink(installation.path(), active_path).is_ok() {
                println!("Swtich version to: {}", &o.version);
            }
        }
        else{
            println!("Unable to find Unity version {}" , &o.version);
            println!("Available versions:");
        }
    }
    else {
        println!("No installed unity versions");
    }
}
