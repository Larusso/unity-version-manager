use std::error::Error;
use std::io;
use std::fmt;
use unity;
use plist;
use serde_json;

#[derive(Debug)]
pub enum UvmError {
    PlistError(plist::Error),
    JsonError(serde_json::Error),
    ParseVersionError(unity::ParseVersionError),
    IoError(io::Error),
}

impl From<io::Error> for UvmError {
    fn from(err: io::Error) -> UvmError {
        UvmError::IoError(err)
    }
}

impl From<plist::Error> for UvmError {
    fn from(err: plist::Error) -> UvmError {
        UvmError::PlistError(err)
    }
}

impl From<serde_json::Error> for UvmError {
    fn from(err: serde_json::Error) -> UvmError {
        UvmError::JsonError(err)
    }
}

impl From<unity::ParseVersionError> for UvmError {
    fn from(err: unity::ParseVersionError) -> UvmError {
        UvmError::ParseVersionError(err)
    }
}

impl Error for UvmError {
    fn description(&self) -> &str {
        match *self {
            UvmError::PlistError(ref err) => err.description(),
            UvmError::JsonError(ref err) => err.description(),
            UvmError::ParseVersionError(ref err) => err.description(),
            UvmError::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(match *self {
            UvmError::PlistError(ref err) => err as &Error,
            UvmError::JsonError(ref err) => err as &Error,
            UvmError::ParseVersionError(ref err) => err as &Error,
            UvmError::IoError(ref err) => err as &Error,
        })
    }
}

impl fmt::Display for UvmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UvmError::PlistError(ref err) => fmt::Display::fmt(err, f),
            UvmError::JsonError(ref err) => fmt::Display::fmt(err, f),
            UvmError::ParseVersionError(ref err) => fmt::Display::fmt(err, f),
            UvmError::IoError(ref err) => fmt::Display::fmt(err, f),
        }
    }
}
