use thiserror::Error;

#[derive(Error, Debug)]
#[error("{msg}")]
pub struct CommandError {
    msg: String,
    code: u32,
    #[source]
    source: anyhow::Error,
}

impl CommandError {
    pub fn new(msg: String, code: u32, source: anyhow::Error) -> Self {
        Self { msg, code, source }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn code(&self) -> u32 {
        self.code
    }
}