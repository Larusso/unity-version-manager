mod v2018;
mod v2019;

pub mod module {
    pub use super::v2018::modules::*;
    pub use super::v2019::modules::*;
}

pub mod manifest {
    pub use super::v2018::manifests::*;
    pub use super::v2019::manifests::*;
}
