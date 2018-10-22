use std::cmp::Ordering;
use std::cmp::PartialOrd;
use std::cmp::Ord;
use super::*;

impl Ord for EditorValue {
    fn cmp(&self, other: &EditorValue) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for EditorValue {
    fn partial_cmp(&self, other: &EditorValue) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
