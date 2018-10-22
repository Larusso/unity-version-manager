use super::*;
use std::convert::From;
use std::convert::Into;
use unity::Installation;

const INSTALLATION_BINARY: &str = "Unity.app";

impl From<Installation> for EditorValue {
    fn from(installation: Installation) -> Self {
        EditorValue {
            version: installation.version().to_owned(),
            location: installation.path().join(INSTALLATION_BINARY),
            manual: true,
        }
    }
}
