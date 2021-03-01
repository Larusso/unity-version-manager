use structopt::clap::arg_enum;

arg_enum! {
    #[derive(PartialEq, Debug, Deserialize)]
    pub enum ColorOption {
        Auto,
        Always,
        Never,
    }
}

impl Default for ColorOption {
    fn default() -> Self {
        Self::Auto
    }
}
