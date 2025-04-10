use anyhow::Result;
use std::collections::HashSet;
use std::fs::remove_dir_all;
use std::io;
use uvm_cli;
use uvm_core::unity;
use uvm_core::unity::Component;
use uvm_core::Version;

use console::style;
use std::process;

use structopt::{
    clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};

const SETTINGS: &'static [AppSettings] = &[
    AppSettings::ColoredHelp,
    AppSettings::DontCollapseArgsInUsage,
];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    /// The api version to uninstall modules or editor
    version: Version,

    /// uninstall all support packages
    #[structopt(short, long)]
    all: bool,

    /// uninstall android support for editor
    #[structopt(long)]
    android: bool,

    /// uninstall ios support for editor
    #[structopt(long)]
    ios: bool,

    /// uninstall webgl support for editor
    #[structopt(long)]
    webgl: bool,

    /// uninstall mobile support (android, ios, webgl)
    #[structopt(long)]
    mobile: bool,

    /// uninstall linux support for editor
    #[cfg(not(target_os = "linux"))]
    #[structopt(long)]
    linux: bool,

    /// uninstall windows support for editor
    #[structopt(long)]
    #[cfg(not(target_os = "windows"))]
    windows: bool,

    /// uninstall macos support for editor
    #[cfg(not(target_os = "macos"))]
    #[structopt(long)]
    macos: bool,

    /// uninstall desktop support (linux, windows)
    #[structopt(long)]
    desktop: bool,

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

impl Opts {
    pub fn install_variants(&self) -> HashSet<Component> {
        let mut variants: HashSet<Component> = HashSet::with_capacity(6);

        if self.android || self.mobile || self.all {
            variants.insert(Component::Android);
        }

        if self.ios || self.mobile || self.all {
            variants.insert(Component::Ios);
        }

        if self.webgl || self.mobile || self.all {
            variants.insert(Component::WebGl);
        }

        #[cfg(not(target_os = "windows"))]
        if self.windows || self.desktop || self.all {
            variants.insert(Component::Windows);
            #[cfg(windows)]
            variants.insert(Component::WindowsIL2CCP);
            variants.insert(Component::WindowsMono);
            variants.insert(Component::WindowsServer);
        }

        #[cfg(not(target_os = "linux"))]
        if self.linux || self.desktop || self.all {
            variants.insert(Component::Linux);
            variants.insert(Component::LinuxIL2CPP);
            variants.insert(Component::LinuxMono);
            variants.insert(Component::LinuxServer);
        }

        #[cfg(not(target_os = "macos"))]
        if self.macos || self.desktop || self.all {
            variants.insert(Component::Mac);
            variants.insert(Component::MacIL2CPP);
            variants.insert(Component::MacMono);
            variants.insert(Component::MacServer);
        }

        if self.all || variants.is_empty() {
            variants.insert(Component::Editor);
            variants.insert(Component::Android);
            variants.insert(Component::Ios);
            variants.insert(Component::WebGl);
            variants.insert(Component::Windows);
            #[cfg(windows)]
            variants.insert(Component::WindowsIL2CCP);
            variants.insert(Component::WindowsMono);
            variants.insert(Component::WindowsServer);
            variants.insert(Component::Linux);
            variants.insert(Component::LinuxIL2CPP);
            variants.insert(Component::LinuxMono);
            variants.insert(Component::LinuxServer);
            variants.insert(Component::Mac);
            variants.insert(Component::MacIL2CPP);
            variants.insert(Component::MacMono);
            variants.insert(Component::MacServer);
        }
        variants
    }
}

fn main() -> std::io::Result<()> {
    let opt = Opts::from_args();

    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

    uninstall(&opt).unwrap_or_else(|_err| {
        let message = "Failure during deinstallation";
        eprintln!("{}", style(message).red());
        process::exit(1);
    });
    eprintln!("{}", style("Finish").green().bold());
    Ok(())
}

fn uninstall(options: &Opts) -> Result<()> {
    let installation = unity::find_installation(&options.version)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "unable to find installation"))?;
    let installed: HashSet<Component> = installation.installed_components().collect();

    let to_uninstall: HashSet<Component> = options.install_variants();

    if to_uninstall.contains(&Component::Editor) {
        eprintln!(
            "{}: {}",
            style("uninstall api version").green(),
            &options.version
        );
        remove_dir_all(installation.path())?
    } else {
        if options.verbose > 0 {
            eprintln!(
                "{}: {}",
                style("uninstall api components").green(),
                &options.version
            );
            eprintln!("{}", style("Components to uninstall:").green());
            for c in &to_uninstall {
                eprintln!("{}", style(c).cyan());
            }

            let mut diff = to_uninstall.difference(&installed).peekable();
            if diff.peek().is_some() {
                eprintln!("");
                eprintln!("{}", style("Skip variants not installed:").yellow());
                for c in diff {
                    eprintln!("{}", style(c).yellow().bold());
                }
            }
        }

        let mut diff = to_uninstall.intersection(&installed).peekable();
        if diff.peek().is_some() {
            eprintln!("Start Uninstall");
            for c in diff {
                if let Some(p) = c.installpath().map(|l| installation.path().join(l)) {
                    eprintln!("Remove {}", c);
                    remove_dir_all(p)?
                }
            }
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "nothing to uninstall").into());
        }
    }
    Ok(())
}
