mod v2017;
mod v2018;
mod v2019;
mod v2020;
mod v2021;
mod v2022;

pub mod module {
    pub use super::v2017::modules::*;
    pub use super::v2018::modules::*;
    pub use super::v2019::modules::*;
    pub use super::v2020::modules::*;
    pub use super::v2021::modules::*;
    pub use super::v2022::modules::*;
}

pub mod manifest {
    pub use super::v2017::manifests::*;
    pub use super::v2018::manifests::*;
    pub use super::v2019::manifests::*;
    pub use super::v2020::manifests::*;
    pub use super::v2021::manifests::*;
    pub use super::v2022::manifests::*;
}
