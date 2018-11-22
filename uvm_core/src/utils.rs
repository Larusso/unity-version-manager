#[cfg(unix)]
use cluFlock::{ExclusiveSliceLock, Flock};
use std::fs::File;
use std::io;

#[cfg(unix)]
pub fn lock_process_or_wait<'a>(lock_file: &'a File) -> io::Result<ExclusiveSliceLock<'a>> {
    match lock_file.try_exclusive_lock() {
        Ok(Some(lock)) => {
            trace!("aquired process lock.");
            Ok(lock)
        }
        Ok(None) => {
            debug!("progress lock already aquired.");
            debug!("wait for other process to finish.");
            let lock = lock_file.exclusive_lock()?;
            Ok(lock)
        }
        Err(err) => Err(err),
    }
}

#[cfg(windows)]
pub fn lock_process_or_wait(_: &File) -> io::Result<()> {
    Ok(())
}
