use plist;
use reqwest;
use serde_ini;
use serde_json;
use std::error::Error;
use std::fmt;
use std::io;
use unity;

#[derive(Debug)]
pub struct IllegalOperationError {
    message: String,
}

impl IllegalOperationError {
    pub fn new(message: &str) -> IllegalOperationError {
        IllegalOperationError {
            message: String::from(message),
        }
    }
}

impl fmt::Display for IllegalOperationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IllegalOperationError {}", self.message)
    }
}

impl Error for IllegalOperationError {
    fn description(&self) -> &str {
        &self.message[..]
    }
}

#[derive(Debug)]
pub enum UvmError {
    PlistError(plist::Error),
    JsonError(serde_json::Error),
    IniError(serde_ini::de::Error),
    ParseVersionError(unity::ParseVersionError),
    IoError(io::Error),
    IllegalOperationError(IllegalOperationError),
    RequestError(reqwest::Error),
    UrlParseError(reqwest::UrlError),
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

impl From<serde_ini::de::Error> for UvmError {
    fn from(err: serde_ini::de::Error) -> UvmError {
        UvmError::IniError(err)
    }
}

impl From<unity::ParseVersionError> for UvmError {
    fn from(err: unity::ParseVersionError) -> UvmError {
        UvmError::ParseVersionError(err)
    }
}

impl From<IllegalOperationError> for UvmError {
    fn from(err: IllegalOperationError) -> UvmError {
        UvmError::IllegalOperationError(err)
    }
}

impl From<reqwest::Error> for UvmError {
    fn from(err: reqwest::Error) -> UvmError {
        UvmError::RequestError(err)
    }
}

impl From<reqwest::UrlError> for UvmError {
    fn from(err: reqwest::UrlError) -> UvmError {
        UvmError::UrlParseError(err)
    }
}

impl Error for UvmError {
    fn description(&self) -> &str {
        match *self {
            UvmError::PlistError(ref err) => err.description(),
            UvmError::JsonError(ref err) => err.description(),
            UvmError::IniError(ref err) => err.description(),
            UvmError::ParseVersionError(ref err) => err.description(),
            UvmError::IoError(ref err) => err.description(),
            UvmError::IllegalOperationError(ref err) => err.description(),
            UvmError::RequestError(ref err) => err.description(),
            UvmError::UrlParseError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(match *self {
            UvmError::PlistError(ref err) => err as &Error,
            UvmError::JsonError(ref err) => err as &Error,
            UvmError::IniError(ref err) => err as &Error,
            UvmError::ParseVersionError(ref err) => err as &Error,
            UvmError::IoError(ref err) => err as &Error,
            UvmError::IllegalOperationError(ref err) => err as &Error,
            UvmError::RequestError(ref err) => err as &Error,
            UvmError::UrlParseError(ref err) => err as &Error,
        })
    }
}

impl fmt::Display for UvmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UvmError::PlistError(ref err) => fmt::Display::fmt(err, f),
            UvmError::JsonError(ref err) => fmt::Display::fmt(err, f),
            UvmError::IniError(ref err) => fmt::Display::fmt(err, f),
            UvmError::ParseVersionError(ref err) => fmt::Display::fmt(err, f),
            UvmError::IoError(ref err) => fmt::Display::fmt(err, f),
            UvmError::IllegalOperationError(ref err) => fmt::Display::fmt(err, f),
            UvmError::RequestError(ref err) => fmt::Display::fmt(err, f),
            UvmError::UrlParseError(ref err) => fmt::Display::fmt(err, f),
        }
    }
}
