use anyhow::{Context, Result};
use console::style;
use log::{info, trace};
use std::fs::OpenOptions;
use std::process;
use uvm_cli;
use uvm_core;
use uvm_core::unity::{Installations, Manifest, Version};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};
use structopt::{clap::crate_authors, clap::crate_description, clap::crate_version, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
struct Opts {
    #[structopt(name = "version", group = "input", required_unless("all"))]
    version: Vec<Version>,

    /// only print moddule to stdout
    #[structopt(long="dry-run")]
    dry_run: bool,

    /// generate modules.json for all installed editors
    #[structopt(short, long, group = "input")]
    all: bool,

    /// print debug output
    #[structopt(short, long)]
    debug: bool,

    /// print more output
    #[structopt(short, long, parse(from_occurrences))]
    verbose: i32,

    /// Color:.
    #[structopt(short, long, possible_values = &ColorOption::variants(), case_insensitive = true, default_value)]
    color: ColorOption,
}

fn generate_for_installation(options: Opts) -> Result<()> {
    let installations = if options.all {
        info!("generate modules.json for all installations");
        uvm_core::list_all_installations()
    } else {
        let v = options
            .version;
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
        let output_path = i.path().join("modules.json");
        if options.dry_run {
            eprintln!("modules json for {}", &i.version());
            eprintln!("output path: {}", output_path.display());
            println!("{}", manifest.modules_json()?);
        } else {
            info!("{}", style(format!("write {}", output_path.display())).green());
            let mut f = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(output_path)?;
            manifest.write_modules_json(&mut f)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let opt = Opts::from_args_safe().map(|opt| {
        set_colors_enabled(&opt.color);
        set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));
        opt
    })?;

    trace!("generate modules.json");

    generate_for_installation(opt).unwrap_or_else(|err| {
        let message = "Unable generate modules.json";
        eprintln!("{}", style(message).red());
        eprintln!("{}", style(err).red());
        process::exit(1);
    });

    eprintln!("{}", style("Done").green());
    Ok(())
}
