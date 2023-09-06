use clap::{ValueEnum, Parser, ArgAction};
use console::Style;
use flexi_logger::{DeferredNow, LogSpecification, Logger};
use log::{Record, Level, LevelFilter};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// print more output
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// print debug output
    #[clap(short, long)]
    pub debug: bool,

    /// Color:.
    #[clap(short, long, value_enum, env = "COLOR_OPTION", default_missing_value("always"), num_args(0..=1), default_value_t = ColorOption::default())]
    pub color: ColorOption,
}

pub fn set_colors_enabled(color: &ColorOption) {
    use ColorOption::*;
    match color {
        Never => console::set_colors_enabled(false),
        Always => console::set_colors_enabled(true),
        Auto => (),
    };
}

pub fn set_loglevel(verbose: i32) {
    let mut log_sepc_builder = LogSpecification::builder();
    let level = match verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::max(),
    };

    log_sepc_builder.default(level);
    let log_spec = log_sepc_builder.build();
    Logger::with(log_spec).format(format_logs).start().unwrap();
}

pub fn format_logs(
    write: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let style = match record.level() {
        Level::Trace => Style::new().white().dim().italic(),
        Level::Debug => Style::new().white().dim(),
        Level::Info => Style::new().green(),
        Level::Warn => Style::new().yellow(),
        Level::Error => Style::new().red(),
    };

    write
        .write(&format!("{}", style.apply_to(record.args())).into_bytes())
        .map(|_| ())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ColorOption {
    Auto,
    Always,
    Never,
}

impl Default for ColorOption {
    fn default() -> Self {
        Self::Auto
    }
}