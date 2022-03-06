use thiserror::Error;
use crate::unity::LocalizationError;

#[derive(Error, Debug)]
pub enum ParseComponentError {
    #[error("unsupported component: {0}")]
    Unsupported(String),
    #[error("unsupported component category: {0}")]
    UnsupportedCategory(String),
    #[error("unsupported locale")]
    UnsupportedLocale {
        #[from]
        source: LocalizationError,
    }
}
