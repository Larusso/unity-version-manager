use console::Style;
use itertools::Itertools;
use log::error;
use uvm_cli::Options;
use uvm_core::error::Result;
use uvm_core::unity::Category;
use uvm_modules;

const USAGE: &str = "
uvm-modules - List available modules for a specified unity version.

Usage:
  uvm-modules [options] [--category=CATEGORY...] <version>
  uvm-modules (-h | --help)

Options:
  -c=CATEGORY, --category=CATEGORY    filter by category
  -s, --show-sync-modules             list also sync modules
  -a, --all                           list also invisible modules
  -v, --verbose                       print more output
  -d, --debug                         print debug output
  --color WHEN                        Coloring: auto, always, never [default: auto]
  -h, --help                          show this help message and exit

Arguments:
  <version>                           The unity version to list modules for in the form of `2018.1.0f3`
";

fn main() {
    list_modules().unwrap_or_else(|err| {
        error!("failed to list modules");
        error!("{}", err);
    })
}

fn list_modules() -> Result<()> {
    let options: uvm_modules::Options = uvm_cli::get_options(USAGE)?;
    let modules = uvm_modules::load_modules(options.version())?;

    if options.debug() {
        println!("{:?}", options);
    }

    let modules = modules
        .iter()
        .filter(|m| options.all() || m.visible)
        .filter(|m| {
            if let Some(c) = options.category() {
                c.contains(&m.category)
            } else {
                true
            }
        })
        .sorted_by(|m_1, m_2| match Ord::cmp(&m_1.category, &m_2.category) {
            std::cmp::Ordering::Equal => Ord::cmp(&m_1.id.to_string(), &m_2.id.to_string()),
            x => x,
        })
        .map(|m| uvm_modules::Module::new(m, &modules))
        .filter(|m| m.parent.is_none() && m.sync.is_none());

    let category_style = Style::new().white().bold();
    let out_style = Style::new().cyan();
    let path_style = Style::new().italic().green();

    let mut category: Option<Category> = None;
    for (i, module) in modules.enumerate() {
        if options.verbose() && (category.is_none() || module.category != category.unwrap()) {
            category = Some(module.category);
            if i != 0 {
                println!();
            }
            println!("{}:", category_style.apply_to(module.category.to_string()));
        }

        print_module(&module, "", 0, options.verbose(), &out_style, &path_style, &options);
    }
    Ok(())
}

fn print_module(
    module: &uvm_modules::Module<'_>,
    prefix: &str,
    rjust: usize,
    verbose: bool,
    out_style: &Style,
    path_style: &Style,
    options: &uvm_modules::Options
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

    if options.show_sync_modules() {
        for sub in module.children().iter().filter(|m| options.all() || m.visible) {
            print_module(
                sub,
                prefix,
                rjust + 2,
                verbose,
                out_style,
                path_style,
                options
            );
        }
    }
}
