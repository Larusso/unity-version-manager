use crate::commands::presentation::{RenderOptions, TextRenderer, as_view_iter};
use clap::Args;
use log::info;
use std::io;
use unity_hub::unity::{
    list_all_installations, list_hub_installations, list_installations, Installation,
};

#[derive(Args, Debug)]
pub struct ListCommand {
    /// print only the path to the current version
    #[arg(short, long = "path")]
    pub path_only: bool,

    /// print more output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// print unity hub installations [default listing]
    #[arg(long = "hub")]
    pub use_hub: bool,

    /// print all unity installations
    #[arg(long)]
    pub all: bool,

    /// print unity installations at default installation location
    #[arg(long)]
    pub system: bool,

    #[arg(short = 'm', long = "modules")]
    pub list_modules: bool,
}

impl ListCommand {
    pub fn execute(&self) -> io::Result<i32> {
        let list_function = if self.system {
            info!("fetch system default installations");
            list_installations
        } else if self.all {
            info!("fetch all installations");
            list_all_installations
        } else if self.use_hub {
            info!("fetch installations from Unity Hub");
            list_hub_installations
        } else {
            info!("fetch installations from Unity Hub");
            list_hub_installations
        };

        match list_function() {
            Ok(installations) => {
                eprintln!("Installed Unity versions:");
                let items: Vec<_> = installations.collect();
                let renderer = TextRenderer::new(RenderOptions { path_only: self.path_only, verbose: self.verbose > 0, list_modules: self.list_modules });
                let rendered = renderer.render_view(&as_view_iter(items));
                eprintln!("{}", rendered);
            }
            Err(e) => {
                eprintln!("Error listing installations: {}", e);
                return Ok(1);
            }
        }

        Ok(0)
    }
}
