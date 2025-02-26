pub mod error;
mod model;
pub use model::*;
mod api;

use crate::error::ErrorRepr;
pub use api::fetch_release::FetchRelease;
pub use api::list_versions::ListVersions;
use unity_version::Version;

pub type Result<T> = std::result::Result<T, error::LivePlatformError>;

pub fn versions() -> Result<impl Iterator<Item = String>> {
    _list_all_versions(false, false)
}

pub fn all_versions() -> Result<impl Iterator<Item = String>> {
    _list_all_versions(false, true)
}

pub fn all_versions_with_revision() -> Result<impl Iterator<Item = String>> {
    _list_all_versions(true, true)
}

fn _list_all_versions(include_revisions: bool, auto_page: bool) -> Result<ListVersions> {
    Ok(ListVersions::builder()
        .include_revision(include_revisions)
        .autopage(auto_page)
        .list()
        .map_err(ErrorRepr::ListVersionsError)?)
}

pub fn fetch_release<V: Into<Version>>(version: V) -> Result<Release> {
    let r = FetchRelease::builder(version)
        .fetch()
        .map_err(ErrorRepr::FetchReleaseError)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result: Vec<String> = all_versions().unwrap().collect();
        println!("{:?}", result)
    }
}
