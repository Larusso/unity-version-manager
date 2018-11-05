use super::Options;

#[derive(Debug, Deserialize)]
pub struct UvmOptions {
    pub arg_command: String,
    pub arg_args: Option<Vec<String>>,
    flag_verbose: bool,
    flag_debug: bool,
}

impl UvmOptions {
    pub fn command(&self) -> &String {
        &self.arg_command
    }

    pub fn take_arguments(&mut self) -> Option<Vec<String>> {
        self.arg_args.take()
    }

    pub fn mut_arguments(&mut self) -> &mut Option<Vec<String>> {
        &mut self.arg_args
    }

    pub fn arguments(&self) -> &Option<Vec<String>> {
        &self.arg_args
    }
}

impl Options for UvmOptions {
    fn verbose(&self) -> bool {
        self.flag_verbose
    }

    fn debug(&self) -> bool {
        self.flag_debug
    }
}
