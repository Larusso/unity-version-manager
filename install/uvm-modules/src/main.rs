use anyhow::{Context, Result};
use console::Style;
use itertools::Itertools;
use std::io;
use std::ops::Deref;
use structopt::{
    clap::crate_authors, clap::crate_description, clap::crate_version, clap::AppSettings, StructOpt,
};
use uvm_cli::{options::ColorOption, set_colors_enabled, set_loglevel};
use uvm_core::unity::{Category, Manifest, Modules, Version};

const SETTINGS: &'static [AppSettings] = &[
    AppSettings::ColoredHelp,
    AppSettings::DontCollapseArgsInUsage,
];

#[derive(StructOpt, Debug)]
#[structopt(version = crate_version!(), author = crate_authors!(), about = crate_description!(), settings = SETTINGS)]
struct Opts {
    /// filter by category
    #[structopt(long, number_of_values = 1)]
    category: Option<Vec<Category>>,

    /// list also sync modules
    #[structopt(long = "show-sync-modules", short)]
    show_sync_modules: bool,

    /// The unity version to list modules for in the form of `2018.1.0f3`
    version: Version,

    /// list also invsible modules
    #[structopt(short, long)]
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

pub struct Module<'a> {
    base: &'a uvm_core::unity::Module,
    children: Vec<Module<'a>>,
}

impl<'a> Deref for Module<'_> {
    type Target = uvm_core::unity::Module;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a> Module<'a> {
    pub fn new(module: &'a uvm_core::unity::Module, lookup: &'a [uvm_core::unity::Module]) -> Self {
        let mut children = Vec::new();
        let base = module;

        for m in lookup.iter() {
            match m.sync {
                Some(id) if id == base.id => children.push(Module::new(m, &lookup)),
                _ => (),
            }
        }

        Module { base, children }
    }

    pub fn children(&self) -> &Vec<Module<'a>> {
        &self.children
    }
}

pub fn load_modules<V: AsRef<Version>>(version: V) -> Result<Modules> {
    let version = version.as_ref();
    let manifest = Manifest::load(version)
        .map_err(|_e| io::Error::new(io::ErrorKind::NotFound, "failed to load manifest"))?;
    let modules: Modules = manifest.into_modules();
    Ok(modules)
}

fn main() -> Result<()> {
    let opt = Opts::from_args();

    set_colors_enabled(&opt.color);
    set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

    list(&opt).context("failed to list Unity modules")?;
    Ok(())
}

fn list(options: &Opts) -> Result<()> {
    let modules = load_modules(&options.version)?;

    let modules = modules
        .iter()
        .filter(|m| options.all || m.visible)
        .filter(|m| {
            if let Some(c) = &options.category {
                c.contains(&m.category)
            } else {
                true
            }
        })
        .sorted_by(|m_1, m_2| match Ord::cmp(&m_1.category, &m_2.category) {
            std::cmp::Ordering::Equal => Ord::cmp(&m_1.id.to_string(), &m_2.id.to_string()),
            x => x,
        })
        .map(|m| Module::new(m, &modules))
        .filter(|m| m.parent.is_none() && m.sync.is_none());

    let category_style = Style::new().white().bold();
    let out_style = Style::new().cyan();
    let path_style = Style::new().italic().green();

    let mut category: Option<Category> = None;
    for (i, module) in modules.enumerate() {
        if (options.verbose >= 1) && (category.is_none() || module.category != category.unwrap()) {
            category = Some(module.category);
            if i != 0 {
                println!();
            }
            println!("{}:", category_style.apply_to(module.category.to_string()));
        }

        print_module(
            &module,
            "",
            0,
            options.verbose >= 1,
            &out_style,
            &path_style,
            &options,
        );
    }
    Ok(())
}

fn print_module(
    module: &Module<'_>,
    prefix: &str,
    rjust: usize,
    verbose: bool,
    out_style: &Style,
    path_style: &Style,
    options: &Opts,
) {
    let p_prefix = console::pad_str(prefix, rjust, console::Alignment::Right, None);
    if verbose {
        println!(
            "{}{} - {}",
            p_prefix,
            out_style.apply_to(module.id),
            path_style.apply_to(module.description.to_string())
        );
    } else {
        println!("{}{}", p_prefix, out_style.apply_to(module.id));
    }

    if options.show_sync_modules {
        for sub in module
            .children()
            .iter()
            .filter(|m| options.all || m.visible)
        {
            print_module(
                sub,
                prefix,
                rjust + 2,
                verbose,
                out_style,
                path_style,
                options,
            );
        }
    }
}
