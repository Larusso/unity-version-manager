use std::result;
use error::UvmError;

pub type Result<T> = result::Result<T, UvmError>;
