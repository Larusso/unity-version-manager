use std::{io, time::Duration};

use clap::Args;
use log::info;
use unity_hub::unity::hub::paths;
use uvm_gc::{GarbageCollector, DEFAULT_MAX_AGE_ENV, DEFAULT_MAX_AGE_HUMAN};



#[derive(Args, Debug)]
pub struct GcCommand {
    /// Execute the garbage collection
    ///
    /// If true, the garbage collection will be executed.
    ///
    /// The default is false.
    #[arg(short, long, default_value_t = false)]
    pub execute: bool,

    /// The maximum age of files to delete
    ///
    /// The maximum age of files to delete. Files older than this will be deleted.
    ///
    /// The default is 4 weeks.
    ///
    /// The format is a human readable duration. e.g. "4w", "30d", "1h", "10m", "30s"
    ///
    /// The default is 4 weeks.
    #[arg(short, long, value_parser = humantime::parse_duration, default_value = DEFAULT_MAX_AGE_HUMAN, env = DEFAULT_MAX_AGE_ENV)]
    pub max_age: Duration,
}

impl GcCommand {
    pub fn execute(&self) -> io::Result<i32> {
        info!("Cleaning up cache");
        let cache_dir = paths::cache_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Unable to determine cache directory")
        })?;

        GarbageCollector::new(cache_dir)
            .with_dry_run(!self.execute)
            .with_max_age(self.max_age)
            .collect()?;

        Ok(0)
    }
}
