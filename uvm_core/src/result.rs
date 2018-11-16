use error::UvmError;
use std::result;

pub type Result<T> = result::Result<T, UvmError>;
