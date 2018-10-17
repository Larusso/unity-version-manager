use std::error::Error;
use std::io;
use std::fmt;
use unity;

#[derive(Debug)]
pub enum UvmError {
    ParseVersionError(unity::ParseVersionError),
    IoError(io::Error),
}

impl From<io::Error> for UvmError {
    fn from(err: io::Error) -> UvmError {
        UvmError::IoError(err)
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
            UvmError::ParseVersionError(ref err) => err.description(),
            UvmError::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(match *self {
            UvmError::ParseVersionError(ref err) => err as &Error,
            UvmError::IoError(ref err) => err as &Error,
        })
    }
}

impl fmt::Display for UvmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UvmError::ParseVersionError(ref err) => fmt::Display::fmt(err, f),
            UvmError::IoError(ref err) => fmt::Display::fmt(err, f),
        }
    }
}
