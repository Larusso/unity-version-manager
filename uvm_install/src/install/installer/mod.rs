use log::{debug, error};
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
pub type InstallerPath = PathBuf;
pub type InstallDestination = PathBuf;
pub type Rename = Option<(PathBuf, PathBuf)>;
#[cfg(windows)]
pub type Cmd = String;
#[cfg(windows)]
pub type OptionalCmd = Option<String>;

#[cfg(target_os = "macos")]
pub type BaseInstaller = (InstallerPath, (), (), Rename);
pub type InstallerWithDestination = (InstallerPath, InstallDestination, (), Rename);

#[cfg(windows)]
pub type InstallerWithDestinationAndOptionalCommand =
    (InstallerPath, InstallDestination, OptionalCmd, Rename);
#[cfg(windows)]
pub type InstallerWithCommand = (InstallerPath, (), Cmd, Rename);
#[cfg(windows)]
pub type InstallerWithOptionalCommand = (InstallerPath, (), OptionalCmd, Rename);

#[cfg(unix)]
mod pkg;
mod po;
mod zip;

#[cfg(unix)]
pub use self::pkg::*;
pub use self::po::*;
pub use self::zip::*;

pub struct Installer<V, T, I> {
    _variant: PhantomData<V>,
    _installer_type: PhantomData<T>,
    inner: I,
}

impl<V, T, X, Y> Installer<V, T, (InstallerPath, X, Y, Rename)> {
    pub fn installer(&self) -> &Path {
        &self.inner.0
    }

    pub fn rename(&self) -> Option<(&Path, &Path)> {
        self.inner
            .3
            .as_ref()
            .map(|(f, t)| (f.as_path(), t.as_path()))
    }
}

impl<V, T, X, Y, Z> Installer<V, T, (X, InstallDestination, Y, Z)> {
    pub fn destination(&self) -> &Path {
        &self.inner.1
    }
}

#[cfg(windows)]
impl<V, T, X, Y, Z> Installer<V, T, (X, Y, Cmd, Z)> {
    pub fn cmd(&self) -> &String {
        &self.inner.2
    }
}

#[cfg(windows)]
impl<V, T, X, Y, Z> Installer<V, T, (X, Y, OptionalCmd, Z)> {
    pub fn cmd(&self) -> Option<&String> {
        self.inner.2.as_ref()
    }
}

impl<V, T> Installer<V, T, InstallerWithDestination> {
    pub fn new<P, D, R>(installer: P, destination: D, rename: Option<(R, R)>) -> Self
    where
        P: AsRef<Path>,
        D: AsRef<Path>,
        R: AsRef<Path>,
    {
        let installer = installer.as_ref().to_path_buf();
        let destination = destination.as_ref().to_path_buf();
        let rename = rename.map(|(f, t)| (f.as_ref().to_path_buf(), t.as_ref().to_path_buf()));

        Installer {
            _variant: PhantomData,
            _installer_type: PhantomData,
            inner: (installer, destination, (), rename),
        }
    }
}

#[cfg(windows)]
impl<V, T> Installer<V, T, InstallerWithDestinationAndOptionalCommand> {
    pub fn new<P, D, R>(
        installer: P,
        destination: D,
        cmd: Option<String>,
        rename: Option<(R, R)>,
    ) -> Self
    where
        P: AsRef<Path>,
        D: AsRef<Path>,
        R: AsRef<Path>,
    {
        let installer = installer.as_ref().to_path_buf();
        let destination = destination.as_ref().to_path_buf();
        let rename = rename.map(|(f, t)| (f.as_ref().to_path_buf(), t.as_ref().to_path_buf()));
        let cmd = cmd;

        Installer {
            _variant: PhantomData,
            _installer_type: PhantomData,
            inner: (installer, destination, cmd, rename),
        }
    }
}

#[cfg(windows)]
impl<V, T> Installer<V, T, InstallerWithCommand> {
    pub fn new<P, R>(installer: P, cmd: String, rename: Option<(R, R)>) -> Self
    where
        P: AsRef<Path>,
        R: AsRef<Path>,
    {
        let installer = installer.as_ref().to_path_buf();
        let cmd = cmd;
        let rename = rename.map(|(f, t)| (f.as_ref().to_path_buf(), t.as_ref().to_path_buf()));

        Installer {
            _variant: PhantomData,
            _installer_type: PhantomData,
            inner: (installer, (), cmd, rename),
        }
    }
}

#[cfg(windows)]
impl<V, T> Installer<V, T, InstallerWithOptionalCommand> {
    pub fn new<P, R>(installer: P, cmd: Option<String>, rename: Option<(R, R)>) -> Self
    where
        P: AsRef<Path>,
        R: AsRef<Path>,
    {
        let installer = installer.as_ref().to_path_buf();
        let cmd = cmd;
        let rename = rename.map(|(f, t)| (f.as_ref().to_path_buf(), t.as_ref().to_path_buf()));

        Installer {
            _variant: PhantomData,
            _installer_type: PhantomData,
            inner: (installer, (), cmd, rename),
        }
    }
}

#[cfg(target_os = "macos")]
impl<V, T> Installer<V, T, BaseInstaller> {
    pub fn new<P, R>(installer: P, rename: Option<(R, R)>) -> Self
    where
        P: AsRef<Path>,
        R: AsRef<Path>,
    {
        let installer = installer.as_ref().to_path_buf();
        let rename = rename.map(|(f, t)| (f.as_ref().to_path_buf(), t.as_ref().to_path_buf()));

        Installer {
            _variant: PhantomData,
            _installer_type: PhantomData,
            inner: (installer, (), (), rename),
        }
    }
}

impl<V, T, I> Installer<V, T, I> {
    pub fn cleanup_file_failable<P: AsRef<Path>>(&self, file: P) {
        let file = file.as_ref();
        if file.exists() && file.is_file() {
            debug!("cleanup file {}", &file.display());
            fs::remove_file(file).unwrap_or_else(|err| {
                error!("failed to cleanup file {}", &file.display());
                error!("{}", err);
            });
        }
    }

    pub fn cleanup_directory_failable<P: AsRef<Path>>(&self, dir: P) {
        let dir = dir.as_ref();
        if dir.exists() && dir.is_dir() {
            debug!("cleanup directory {}", dir.display());
            fs::remove_dir_all(dir).unwrap_or_else(|err| {
                error!("failed to cleanup directory {}", dir.display());
                error!("{}", err);
            })
        }
    }
}
