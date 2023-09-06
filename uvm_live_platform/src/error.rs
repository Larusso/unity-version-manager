use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug)]
pub struct LivePlatformError {
    pub msg: String,
    
    #[source]
    pub source: anyhow::Error,
}

impl Display for LivePlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LivePlatformError: {}", self.msg)
    }
}
