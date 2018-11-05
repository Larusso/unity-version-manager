pub mod cask;
pub mod tap;

use std::io;
use std::process::{Command, Stdio};

pub fn update() -> io::Result<()> where
{
    Command::new("brew")
        .arg("update")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
        Ok(())
}
