pub mod v1;
pub mod v2;

mod ini;
mod md5;
mod client;

pub type ComponentData = ini::IniData;

pub use self::v1::{Manifest, ManifestIteratorItem};
pub use self::md5::MD5;
pub use self::ini::IniManifest;
