use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt, str::FromStr};
use thiserror::Error;

#[derive(PartialEq, Eq, Ord, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReleaseType {
    Alpha,
    Beta,
    Patch,
    Final,
}

impl PartialOrd for ReleaseType {
    fn partial_cmp(&self, other: &ReleaseType) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            match *self {
                ReleaseType::Final => write!(f, "final"),
                ReleaseType::Patch => write!(f, "patch"),
                ReleaseType::Beta => write!(f, "beta"),
                ReleaseType::Alpha => write!(f, "alpha"),
            }
        } else {
            match *self {
                ReleaseType::Final => write!(f, "f"),
                ReleaseType::Patch => write!(f, "p"),
                ReleaseType::Beta => write!(f, "b"),
                ReleaseType::Alpha => write!(f, "a"),
            }
        }
    }
}

impl Default for ReleaseType {
    fn default() -> ReleaseType {
        ReleaseType::Final
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ReleaseTypeError {
    #[error("Unknown release type: {0}")]
    UnknownType(String),

    #[error("Invalid release type character: {0}")]
    InvalidCharacter(char),
}

impl FromStr for ReleaseType {
    type Err = ReleaseTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" => Ok(ReleaseType::Alpha),
            "b" => Ok(ReleaseType::Beta),
            "p" => Ok(ReleaseType::Patch),
            "f" => Ok(ReleaseType::Final),
            _ => Err(ReleaseTypeError::UnknownType(s.to_string())),
        }
    }
}

impl TryFrom<char> for ReleaseType {
    type Error = ReleaseTypeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'f' => Ok(ReleaseType::Final),
            'b' => Ok(ReleaseType::Beta),
            'a' => Ok(ReleaseType::Alpha),
            'p' => Ok(ReleaseType::Patch),
            _ => Err(ReleaseTypeError::InvalidCharacter(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    #[test]
    fn should_return_final_when_given_f_as_input() {
        let result = ReleaseType::try_from('f');
        assert_eq!(result, Ok(ReleaseType::Final));
    }
    #[test]
    fn should_return_alpha_when_given_a_as_input() {
        let result = ReleaseType::try_from('a');
        assert_eq!(result, Ok(ReleaseType::Alpha));
    }
    #[test]
    fn should_return_beta_when_given_b_as_input() {
        let result = ReleaseType::try_from('b');
        assert_eq!(result, Ok(ReleaseType::Beta));
    }
    #[test]
    fn should_return_patch_when_given_p_as_input() {
        let result = ReleaseType::try_from('p');
        assert_eq!(result, Ok(ReleaseType::Patch));
    }
    #[test]
    fn should_return_error_when_given_empty_input() {
        let result = ReleaseType::try_from(' ');
        assert_eq!(result, Err(ReleaseTypeError::InvalidCharacter(' ')));
    }

    #[test]
    fn should_return_correct_to_string_value() {
        assert_eq!(ReleaseType::Final.to_string(), "f");
        assert_eq!(ReleaseType::Patch.to_string(), "p");
        assert_eq!(ReleaseType::Alpha.to_string(), "a");
        assert_eq!(ReleaseType::Beta.to_string(), "b");
    }
    #[test]
    fn should_format_using_correct_alternative_format() {
        assert_eq!(&format!("{:#}", ReleaseType::Final), "final");
        assert_eq!(&format!("{:#}", ReleaseType::Patch), "patch");
        assert_eq!(&format!("{:#}", ReleaseType::Beta), "beta");
        assert_eq!(&format!("{:#}", ReleaseType::Alpha), "alpha");
    }

    proptest! {
        #[test]
        fn from_str_does_not_crash(s in "\\PC*") {
            let _r = ReleaseType::from_str(&s);
        }

        #[test]
        fn from_str_supports_all_valid_cases(s in "(a|b|f|p)") {
            ReleaseType::from_str(&s).unwrap();
        }
    }
}
