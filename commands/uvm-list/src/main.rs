use anyhow::{Context, Result};
use console::Style;
use log::info;
use std::io;
use structopt::{
    clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli;
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};

const SETTINGS: &'static [AppSettings] = &[
    AppSettings::ColoredHelp,
    AppSettings::DontCollapseArgsInUsage,
];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    /// print only the path to the current version
    #[structopt(short, long = "path")]
    path_only: bool,

    /// print more output
    #[structopt(short, long, parse(from_occurrences))]
    verbose: i32,

    /// print debug output
    #[structopt(short, long)]
    debug: bool,

    /// print unity hub installations [default listing]
    #[structopt(long = "hub")]
    use_hub: bool,

    /// print all unity installations
    #[structopt(long)]
    all: bool,

    /// print unity installations at default installation location
    #[structopt(long)]
    system: bool,

    #[structopt(short = "m", long = "modules")]
    list_modules: bool,

    /// Color:.
    #[structopt(short, long, possible_values = &ColorOption::variants(), case_insensitive = true, default_value)]
    color: ColorOption,
}

fn main() -> Result<()> {
    let opt = Opts::from_args();

    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

    list(&opt).context("failed to list Unity installations")?;
    Ok(())
}

fn list(options: &Opts) -> io::Result<()> {
    let current_version = uvm_core::current_installation().ok();
    let list_function = if options.system {
        info!("fetch system installations");
        uvm_core::list_installations
    } else if options.all {
        info!("fetch all installations");
        uvm_core::list_all_installations
    } else if options.use_hub {
        info!("fetch installations from unity hub");
        uvm_core::list_hub_installations
    } else {
        info!("fetch installations from unity hub");
        uvm_core::list_hub_installations
    };

    if let Ok(installations) = list_function() {
        eprintln!("Installed Unity versions:");
        let verbose = options.verbose;
        let path_only = options.path_only;

        let output = installations.fold(String::new(), |out, installation| {
            let mut out_style = Style::new().cyan();
            let mut path_style = Style::new().italic().green();

            if let Some(ref current) = &current_version {
                if current == &installation {
                    out_style = out_style.yellow().bold();
                    path_style = path_style.italic().yellow();
                }
            }
            let mut new_line = out;

            if !path_only {
                new_line += &format!("{}", out_style.apply_to(installation.version().to_string()));
            }

            if verbose > 0 {
                new_line += " - ";
            }

            if verbose > 0 || path_only {
                new_line += &format!("{}", path_style.apply_to(installation.path().display()));
            }
            new_line += "\n";
            if options.list_modules {
                for c in installation.installed_components() {
                    new_line += &format!("  {}\n", out_style.apply_to(c));
                }
            }
            new_line
        });

        eprintln!("{}", &output);
    };

    Ok(())
}
