use console::Style;
use itertools::Itertools;
use log::error;
use uvm_cli::Options;
use uvm_core::error::Result;
use uvm_core::unity::Module;
use uvm_modules;
use uvm_core::unity::Category;

const USAGE: &str = "
uvm-modules - List available modules for a specified unity version.

Usage:
  uvm-modules [options] [--category=CATEGORY...] <version>
  uvm-modules (-h | --help)

Options:
  -c=CATEGORY, --category=CATEGORY    filter by category
  -v, --verbose                       print more output
  -d, --debug                         print debug output
  --color WHEN                        Coloring: auto, always, never [default: auto]
  -h, --help                          show this help message and exit

Arguments:
  <version>              The unity version to list modules for in the form of `2018.1.0f3`
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

    let modules: Vec<Module> = modules
        .into_iter()
        .filter(|m| m.parent.is_none() && m.sync.is_none())
        .filter(|m| {
            if let Some(c) = options.category() {
                c.contains(&m.category)
            } else {
                true
            }
        })
        .sorted_by(|m_1, m_2| Ord::cmp(&m_1.category, &m_2.category))
        .collect();

    let category_style = Style::new().white().bold();
    let out_style = Style::new().cyan();
    let path_style = Style::new().italic().green();

    let mut category:Option<Category> = None;
    for (i,module) in modules.into_iter().enumerate() {
        if options.verbose() {
            if category.is_none() || module.category != category.unwrap() {
                category = Some(module.category);
                if i != 0 {
                    println!("");
                }
                println!("{}:", category_style.apply_to(module.category.to_string()));
            }
            println!(
                "{} - {}",
                out_style.apply_to(module.id),
                path_style.apply_to(module.description)
            );
        } else {
            println!("{}", out_style.apply_to(module.id));
        }
    }
    Ok(())
}
