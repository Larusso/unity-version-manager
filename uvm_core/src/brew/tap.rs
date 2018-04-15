use std::fs;
use std::io;
use std::process::Command;

const BREW_TAPS_LOCATION: &'static str = "/usr/local/Homebrew/Library/Taps";

pub struct Taps(Box<Iterator<Item = String>>);

impl Taps {
    fn new() -> io::Result<Taps> {
        let read_dir = fs::read_dir(BREW_TAPS_LOCATION)?;
        let iter = read_dir
            .filter_map(io::Result::ok)
            .flat_map(|d| {
                let inner_read = fs::read_dir(d.path()).expect("read dir");
                inner_read.filter_map(io::Result::ok)
            })
            .map(|d| {
                let path = d.path();
                let parent = path.parent()
                    .and_then(|d| d.file_name())
                    .and_then(|d| d.to_str());

                let tap_name = path.file_name()
                    .and_then(|d| d.to_str())
                    .and_then(|f| Some(f.replace("homebrew-","")));

                match (parent, tap_name) {
                    (Some(p), Some(t)) => Some(format!("{}/{}", p, t)),
                    _ => None
                }
            }).filter_map(|d| d );
        Ok(Taps(Box::new(iter)))
    }
}

impl Iterator for Taps {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub fn list() -> io::Result<Taps> {
    Taps::new()
}

pub fn contains(tap_name: &str) -> bool {
    if let Ok(l) = list() {
        return l.collect::<Vec<String>>().contains(&String::from(tap_name))
    }
    false
}

pub fn add(tap_name: &str) -> io::Result<()> {
    let output = Command::new("brew").args(&["tap", tap_name]).output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!(
            "failed to add tap:/n{}",
            String::from_utf8_lossy(&output.stderr)
        ),
    ))
}

pub fn ensure(tap_name: &str) -> io::Result<()> {
    if !contains(tap_name) {
        return add(tap_name);
    }
    Ok(())
}
