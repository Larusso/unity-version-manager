#[cfg(unix)]
use cluFlock::{ExclusiveFlock, FlockLock};
use std::fs::File;
use std::io;

#[cfg(unix)]
pub fn lock_process_or_wait<'a>(lock_file: &'a File) -> io::Result<FlockLock<&'a File>> {
    match lock_file.try_lock() {
        Ok(lock) => {
            trace!("aquired process lock.");
            Ok(lock)
        }
        Err(_) => {
            debug!("progress lock already aquired.");
            debug!("wait for other process to finish.");
            let lock = lock_file.wait_lock()?;
            Ok(lock)
        }
        //Err(err) => Err(err),
    }
}

#[cfg(windows)]
pub fn lock_process_or_wait(_: &File) -> io::Result<()> {
    Ok(())
}
