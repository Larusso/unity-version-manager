use std::io;
use std::process::Command;
use std::process::Child;
use std::str;

pub type Cask = String;
pub type Casks = Vec<Cask>;

pub fn list() -> io::Result<Casks> {
    Command::new("brew")
        .arg("tap")
        .output()
        .map(|o| o.stdout)
        .and_then(|stdout| {
            let out = str::from_utf8(&stdout)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            Ok(out.lines().map(|line| Cask::from(line)).collect())
        })
}

pub fn install(cask: &str) -> io::Result<Child> {
    Command::new("brew")
        .arg("cask")
        .arg("install")
        .arg(cask.trim())
        .spawn()
}
