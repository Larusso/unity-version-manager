use std::path::PathBuf;
use unity::Version;
use std::cmp::Ordering;

#[derive(PartialEq, Eq, Debug)]
pub struct Installation {
    pub version: Version,
    pub path: PathBuf,
}

impl Ord for Installation {
    fn cmp(&self, other: &Installation) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for Installation {
    fn partial_cmp(&self, other: &Installation) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
