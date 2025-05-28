use thiserror::Error;
use derive_more::{Deref, Display};
use std::str::FromStr;

#[derive(Debug, Error)]
pub enum RevisionHashError {
    #[error("Input must be exactly 12 characters long")]
    InvalidLength,

    #[error("Input contains invalid characters")]
    InvalidCharacter,
}

#[derive(Eq, Debug, Clone, Hash, Display, Deref)]
#[display(fmd = "{}", 0)]
pub struct RevisionHash(String);

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
        self.as_str() == other.as_str()
    }
}

impl FromStr for RevisionHash {
    type Err = RevisionHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
       RevisionHash::new(s) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_hash_creation() {
        let hash = RevisionHash::new("0123456789ab").unwrap();
        assert_eq!(hash.as_str(), "0123456789ab");
    }

    #[test]
    fn test_invalid_length() {
        assert!(matches!(
            RevisionHash::new("123456"),
            Err(RevisionHashError::InvalidLength)
        ));
        assert!(matches!(
            RevisionHash::new("0123456789abcd"),
            Err(RevisionHashError::InvalidLength)
        ));
    }

    #[test]
    fn test_invalid_characters() {
        assert!(matches!(
            RevisionHash::new("0123456789xy"),
            Err(RevisionHashError::InvalidCharacter)
        ));
    }

    #[test]
    fn test_string_parsing() {
        let hash: Result<RevisionHash, _> = "0123456789ab".parse();
        assert!(hash.is_ok());
        assert_eq!(hash.unwrap().as_str(), "0123456789ab");
    }

    #[test]
    fn test_equality() {
        let hash1 = RevisionHash::new("0123456789ab").unwrap();
        let hash2 = RevisionHash::new("0123456789ab").unwrap();
        let hash3 = RevisionHash::new("fedcba987654").unwrap();

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}

