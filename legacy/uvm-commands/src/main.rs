use anyhow::Result;
use console::Style;
use console::Term;
use uvm_cli;

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
  /// print a list with commands
  #[structopt(short, long)]
  list: bool,

  /// print single column list
  #[structopt(short = "1")]
  single_column: bool,

  /// print only the path to the commands
  #[structopt(short, long = "path")]
  path_only: bool,

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

fn main() -> Result<()> {
  let opt = Opts::from_args();

  set_colors_enabled(&opt.color);
  set_loglevel(opt.debug.then(|| 2).unwrap_or(opt.verbose));

  let commands = uvm_cli::find_sub_commands()?;
  let out_style = Style::new().cyan();
  let path_style = Style::new().italic().green();

  let list = opt.list || opt.single_column || opt.verbose > 0;
  let path_only = opt.path_only;
  let single_column = opt.single_column;

  let seperator = if list || !Term::stdout().is_term() {
    "\n"
  } else {
    " "
  };
  let output = commands.fold(String::new(), |out, command| {
    let mut new_line = out;

    if !path_only || (list && !single_column) {
      new_line += &format!("{}", out_style.apply_to(command.command_name().to_string()));
    }

    if list && !single_column {
      new_line += " - ";
    }

    if path_only || (list && !single_column) {
      new_line += &format!("{}", path_style.apply_to(command.path().display()));
    }
    new_line += seperator;
    new_line
  });
  eprintln!("{}", &output);

  Ok(())
}
