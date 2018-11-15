#[cfg(unix)]
use cluFlock::{Flock, ExclusiveSliceLock};
use std::io;
use std::fs::File;

#[cfg(unix)]
pub fn lock_process_or_wait<'a>(lock_file:&'a File) -> io::Result<ExclusiveSliceLock<'a>> {
    match lock_file.try_exclusive_lock() {
        Ok(Some(lock)) => {
            trace!("aquired process lock.");
            Ok(lock)
        },
        Ok(None) => {
            debug!("progress lock already aquired.");
            debug!("wait for other process to finish.");
            let lock = lock_file.exclusive_lock()?;
            Ok(lock)
        },
        Err(err) => Err(err)
    }
}

#[cfg(windows)]
pub fn lock_process_or_wait(lock_file:File) -> io::Result<()> {
    Ok(())
}
