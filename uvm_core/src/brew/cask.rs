use std::io;
use std::process::Command;
use std::process::Child;
use std::str;
use std::fs;

pub type Cask = String;
pub struct Casks(Box<Iterator<Item = Cask>>);

const BREW_CASKS_LOCATION: &'static str = "/usr/local/Caskroom";

impl Casks {
    fn new() -> io::Result<Casks> {
        let read_dir = fs::read_dir(BREW_CASKS_LOCATION)?;
        let iter = read_dir
            .filter_map(io::Result::ok)
            .map(|e| {
                e.path()
                .file_name()
                .and_then(|f| f.to_str())
                .and_then(|s| Some(String::from(s)))
            }).filter_map(|d| d);
        Ok(Casks(Box::new(iter)))
    }
}

impl Iterator for Casks {
    type Item = Cask;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub fn list() -> io::Result<Casks> {
    Casks::new()
}

pub fn install(cask: &str) -> io::Result<Child> {
    Command::new("brew")
        .arg("cask")
        .arg("install")
        .arg(cask.trim())
        .spawn()
}
