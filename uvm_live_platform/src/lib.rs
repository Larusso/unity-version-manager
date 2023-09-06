pub mod error;
mod model;
pub use model::*;
mod api;

pub use api::list_versions::ListVersions;

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

fn _list_all_versions(include_revisions: bool, autopage: bool) -> Result<ListVersions> {
    ListVersions::builder().include_revision(include_revisions).autopage(autopage).list()
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
