use clap::Args;
use log::info;
use uvm_detect::DetectOptions;
use std::path::PathBuf;
use std::{env, io};

use crate::commands::Command;

#[derive(Args, Debug)]
pub struct DetectCommand {
    pub project_path: Option<PathBuf>,

    #[arg(short, long)]
    pub recursive: bool,
}

impl Command for DetectCommand {
    fn execute(&self) -> io::Result<i32> {
        let project_path = match self.project_path.as_ref() {
            Some(p) => p,
            _ => &env::current_dir()?,
        };
        
        info!("Detect the project version at path {}", project_path.display());
        let version = DetectOptions::new().recursive(self.recursive).detect_project_version(project_path)?; 
        println!("{}", version);
        Ok(0)
    }
}
