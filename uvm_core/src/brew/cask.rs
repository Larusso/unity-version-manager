use std::io;
use std::process::{Command, Stdio};
use std::process::Child;
use std::str;
use std::fs;
use std::ffi::OsStr;

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

pub fn install<I, S>(casks: I) -> io::Result<Child> where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>
{
    Command::new("brew")
        .arg("cask")
        .arg("install")
        .args(casks)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub fn fetch<I, S>(casks: I, force:bool) -> io::Result<Child> where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>
{
    Command::new("brew")
        .arg("cask")
        .arg("fetch")
        .args(casks)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub fn uninstall<I, S>(casks: I) -> io::Result<Child> where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>
{
    Command::new("brew")
        .arg("cask")
        .arg("uninstall")
        .arg("--force")
        .args(casks)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub fn search(pattern: &str) -> io::Result<Child> {
    Command::new("brew")
        .arg("search")
        .arg("--casks")
        .arg(pattern)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
