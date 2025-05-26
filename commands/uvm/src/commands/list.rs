use clap::Args;

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(short, long = "path")]
    pub path_only: bool,

    #[arg(long = "hub")]
    pub use_hub: bool,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub system: bool,

    #[arg(short = 'm', long = "modules")]
    pub list_modules: bool,
}