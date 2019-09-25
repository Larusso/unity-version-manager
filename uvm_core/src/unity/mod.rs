mod component;
mod current_installation;
pub mod hub;
mod installation;
pub mod urls;
mod version;
mod localization;

use core::iter::FromIterator;
pub use self::component::Component;
pub use self::localization::Localization;
pub use self::current_installation::current_installation;
pub use self::current_installation::CurrentInstallation;
pub use self::installation::Installation;
pub use self::version::all_versions;
pub use self::version::manifest::Manifest;
pub use self::version::manifest::ManifestIteratorItem;
pub use self::version::manifest::MD5;
pub use self::version::manifest::ComponentData;
pub use self::version::module::{Module, Modules, ModulesMap};
pub use self::version::Version;
pub use self::version::VersionType;
pub use self::version::{UvmVersionError, UvmVersionErrorKind, ResultExt as UvmVersionErrorResultExt, Result as UvmVersionErrorResult};

use crate::error::*;
use itertools::Itertools;
use std::convert::From;
use std::fs;
use std::io;
use std::path::Path;
use std::slice::Iter;

pub struct Installations(Box<dyn Iterator<Item = Installation>>);
pub struct Versions(Box<dyn Iterator<Item = Version>>);

pub struct InstalledComponents {
    installation: Installation,
    components: Iter<'static, Component>,
}

impl Installations {
    fn new(install_location: &Path) -> Result<Installations> {
        debug!(
            "fetch unity installations from {}",
            install_location.display()
        );
        let read_dir = fs::read_dir(install_location)?;

        let iter = read_dir
            .filter_map(|dir_entry| dir_entry.ok())
            .map(|entry| entry.path())
            .map(Installation::new)
            .filter_map(Result::ok);
        Ok(Installations(Box::new(iter)))
    }

    fn empty() -> Installations {
        Installations(Box::new(::std::iter::empty()))
    }

    pub fn versions(self) -> Versions {
        self.into()
    }
}

impl From<hub::editors::Editors> for Installations {
    fn from(editors: hub::editors::Editors) -> Self {
        let iter = editors.into_iter().map(Installation::from);
        Installations(Box::new(iter))
    }
}

impl Iterator for Installations {
    type Item = Installation;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Iterator for Versions {
    type Item = Version;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl InstalledComponents {
    pub fn new(installation: Installation) -> InstalledComponents {
        InstalledComponents {
            installation,
            components: Component::iterator(),
        }
    }
}

impl Iterator for InstalledComponents {
    type Item = Component;

    fn next(&mut self) -> Option<Self::Item> {
        for c in &mut self.components {
            if c.is_installed(&self.installation.path()) {
                trace!(
                    "found component {:?} installed at {}",
                    &c,
                    &c.install_location().unwrap().display()
                );
                return Some(*c);
            }
        }
        None
    }
}

impl From<Installations> for Versions {
    fn from(installations: Installations) -> Self {
        let iter = installations.map(|i| i.version_owned());
        Versions(Box::new(iter))
    }
}

impl FromIterator<Installation> for Installations {
    fn from_iter<I: IntoIterator<Item=Installation>>(iter: I) -> Self {
        let c:Vec<Installation> = iter.into_iter().collect();
        Installations(Box::new(c.into_iter()))
    }
}

pub fn list_all_installations() -> Result<Installations> {
    let i1 = list_installations()?;
    let i2 = hub::list_installations()?;
    let iter = i1.chain(i2);
    let unique = iter.unique_by(|installation| installation.version().to_owned());
    Ok(Installations(Box::new(unique)))
}

pub fn list_installations() -> Result<Installations> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let application_path = dirs_2::application_dir();

    #[cfg(target_os = "linux")]
    let application_path = dirs_2::executable_dir();

    application_path.ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "unable to locate application_dir").into()
    })
    .and_then(|application_dir| {
        list_installations_in_dir(&application_dir)
        .or_else(|err| {
            match err {
                UvmError(UvmErrorKind::Io(ref io_error),_) => {
                    io_error.raw_os_error()
                        .and_then(|error| {
                            match error {
                                2 => {
                                    warn!("{}", io_error);
                                    Some(Installations::empty())
                                },
                                _ => None
                            }
                        })
                },
                _ => None
            }.ok_or_else(|| err)
        })
    })
}

pub fn list_hub_installations() -> Result<Installations> {
    hub::list_installations().map_err(|err| err.into())
}

pub fn list_installations_in_dir(install_location: &Path) -> Result<Installations> {
    Installations::new(install_location)
}

pub fn find_installation(version: &Version) -> Result<Installation> {
    list_all_installations().and_then(|mut installations| {
        installations
            .find(|installation| installation.version() == version)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("unable to locate installation with version {}", version),
                ).into()
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use plist::serde::serialize_to_xml;
    use std::fs;
    use std::fs::File;
    use std::path::Path;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::Builder;
    use crate::unity::installation::AppInfo;

    fn create_test_path(base_dir: &PathBuf, version: &str) -> PathBuf {
        let path = base_dir.join(format!("Unity-{version}", version = version));
        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&path).unwrap();

        let info_plist_path = path.join(Path::new("Unity.app/Contents/Info.plist"));
        dir_builder
            .create(info_plist_path.parent().unwrap())
            .unwrap();

        let info = AppInfo {
            c_f_bundle_version: String::from_str(version).unwrap(),
            unity_build_number: String::from_str("ssdsdsdd").unwrap(),
        };
        let file = File::create(info_plist_path).unwrap();
        serialize_to_xml(file, &info).unwrap();

        path
    }

    macro_rules! prepare_unity_installations {
        ($($input:expr),*) => {
            {
                let test_dir = Builder::new()
                                .prefix("list_installations")
                                .rand_bytes(5)
                                .suffix("_in_directory")
                                .tempdir()
                                .unwrap();
                {
                    $(
                        create_test_path(&test_dir.path().to_path_buf(), $input);
                    )*
                }
                test_dir
            }
        };
    }

    #[test]
    fn list_installations_in_directory_filters_unity_installations() {
        let test_dir = prepare_unity_installations!["2017.1.2f3", "2017.2.3f4"];

        let mut builder = Builder::new();
        builder.prefix("some-dir");
        builder.rand_bytes(5);

        let _temp_dir1 = builder.tempdir_in(&test_dir).unwrap();
        let _temp_dir2 = builder.tempdir_in(&test_dir).unwrap();
        let _temp_dir3 = builder.tempdir_in(&test_dir).unwrap();

        let subject = Installations::new(test_dir.path()).unwrap();

        let r1 = Version::from_str("2017.1.2f3").unwrap();
        let r2 = Version::from_str("2017.2.3f4").unwrap();

        assert_eq!(fs::read_dir(&test_dir).unwrap().count(), 5);

        for installation in subject {
            let version = installation.version_owned();
            assert!(version == r1 || version == r2);
        }
    }

    #[test]
    fn list_installations_in_empty_directory_returns_no_error() {
        let test_dir = prepare_unity_installations![];
        assert!(Installations::new(test_dir.path()).is_ok());
    }

    // #[test]
    // fn list_installations_returns_empty_iterator_when_dir_does_not_exist() {
    //     assert_eq!(list_installations().unwrap().count(), 0);
    // }

    #[test]
    fn installations_can_be_converted_to_versions() {
        let test_dir = prepare_unity_installations!["2017.1.2f3", "2017.2.3f4"];

        let installations = Installations::new(test_dir.path()).unwrap();
        let subject = installations.versions();

        let r1 = Version::from_str("2017.1.2f3").unwrap();
        let r2 = Version::from_str("2017.2.3f4").unwrap();
        for version in subject {
            assert!(version == r1 || version == r2);
        }
    }

    #[test]
    fn versions_can_be_created_from_installations() {
        let test_dir = prepare_unity_installations!["2017.1.2f3", "2017.2.3f4"];

        let installations = Installations::new(test_dir.path()).unwrap();
        let subject = Versions::from(installations);

        let r1 = Version::from_str("2017.1.2f3").unwrap();
        let r2 = Version::from_str("2017.2.3f4").unwrap();
        for version in subject {
            assert!(version == r1 || version == r2);
        }
    }

    proptest! {
        #[test]
        fn doesnt_crash(ref s in "\\PC*") {
            let _ = Installations::new(Path::new(s));
        }

        #[test]
        #[cfg(targed_os="macos")]
        fn parses_all_valid_versions(ref s in r"[0-9]{1,4}\.[0-9]{1,4}\.[0-9]{1,4}[fpb][0-9]{1,4}") {
            let test_dir = prepare_unity_installations![
                s
            ];
            let mut subject = Installations::new(test_dir.path()).unwrap();
            assert_eq!(subject.count(), 1);
        }
    }
}
