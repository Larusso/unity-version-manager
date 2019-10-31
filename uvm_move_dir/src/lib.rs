use log::*;
use std::fs;
use std::io;
use std::path::Path;
use tempfile::tempdir;

#[cfg(windows)]
mod win_move_file;

fn visit_dirs<P: AsRef<Path>>(
    dir: P,
    cb: &mut dyn for<'r> std::ops::FnMut(&'r fs::DirEntry),
) -> io::Result<()> {
    let dir = dir.as_ref();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn recursive_file_count<P: AsRef<Path>>(dir: P) -> io::Result<u64> {
    let mut count = 0;
    visit_dirs(dir, &mut |_| count += 1)?;
    Ok(count)
}

pub fn move_dir<S: AsRef<Path>, D: AsRef<Path>>(source: S, destination: D) -> io::Result<()> {
    let destination = destination.as_ref();
    let source = source.as_ref();

    if !source.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "from parameter must be a directory"));
    }

    if source.starts_with(destination) {
        trace!("attempt to move files some levels up!");
        let tempdir = tempdir()?;
        let sub = tempdir.path().join("sub");
        #[cfg(unix)]
        fs::DirBuilder::new().create(&sub)?;
        #[cfg(unix)]
        fs::rename(source, &sub)?;
        #[cfg(windows)]
        win_move_file::rename(source, &sub)?;

        move_dir(&sub, destination).map_err(|err|{
            match fs::rename(&sub,source) {
                Err(err) => err,
                _ => err
            }
        })?;
    } else {
        trace!("rename dir");
        if destination.exists() {
            if destination.is_dir() && recursive_file_count(destination)? == 0 {
                fs::remove_dir_all(destination)?;
                move_dir(source, destination)?;
            }
            else {
                return Err(io::Error::from(io::ErrorKind::AlreadyExists));
            }
        } else {
            #[cfg(unix)]
            fs::DirBuilder::new()
                .recursive(true)
                .create(destination)?;
            #[cfg(unix)]
            fs::rename(source, destination)?;
            #[cfg(windows)]
            win_move_file::rename(source, destination)?;
        }
    }

    Ok(())
}
