use std::path::PathBuf;

pub mod unity;
pub fn cache_dir() -> Option<PathBuf> {
    dirs_2::cache_dir().map(|path| path.join("com.github.larusso.unity-version-manager"))
}

pub fn locks_dir() -> Option<PathBuf> {
    cache_dir().map(|path| path.join("locks"))
}

pub fn hash_cache_dir() -> Option<PathBuf> {
    cache_dir().map(|path| path.join("versions"))
}
