use super::*;
use std::convert::From;
use std::convert::Into;
use unity::Installation;

const INSTALLATION_BINARY: &str = "Unity.app";

impl From<Installation> for EditorInstallation {
    fn from(installation: Installation) -> Self {
        EditorInstallation {
            version: installation.version().to_owned(),
            location: installation.path().join(INSTALLATION_BINARY),
            manual: true,
        }
    }
}
