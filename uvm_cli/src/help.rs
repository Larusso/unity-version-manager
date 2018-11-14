#[derive(Debug, Deserialize)]
pub struct HelpOptions {
    pub arg_command: String,
}

impl HelpOptions {
    pub fn command(&self) -> &String {
        &self.arg_command
    }
}

impl super::Options for HelpOptions {}
