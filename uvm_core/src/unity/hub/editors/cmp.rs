use std::cmp::Ordering;
use std::cmp::PartialOrd;
use std::cmp::Ord;
use super::*;

impl Ord for EditorInstallation {
    fn cmp(&self, other: &EditorInstallation) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for EditorInstallation {
    fn partial_cmp(&self, other: &EditorInstallation) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
