use thiserror::Error;
use derive_more::Deref;
use std::str::FromStr;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum RevisionHashError {
    #[error("Input must be exactly 12 characters long")]
    InvalidLength,

    #[error("Input contains invalid characters")]
    InvalidCharacter,
}

#[derive(Eq, Debug, Clone, Hash, Deref)]
#[allow(dead_code)]
pub struct RevisionHash(String);

impl std::fmt::Display for RevisionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(dead_code)]
impl RevisionHash {
    pub fn new(input: &str) -> Result<Self, RevisionHashError> {
        if input.len() != 12 {
            return Err(RevisionHashError::InvalidLength);
        }

        if input.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(RevisionHash(input.to_string()))
        } else {
            return Err(RevisionHashError::InvalidCharacter);
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq for RevisionHash {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.0
    }
}

impl FromStr for RevisionHash {
    type Err = RevisionHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
       RevisionHash::new(s) 
    }
}