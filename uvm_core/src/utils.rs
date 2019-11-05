#[cfg(unix)]
use cluFlock::{ExclusiveFlock, FlockLock};
use reqwest::Client;
use reqwest::header::{USER_AGENT, CONTENT_DISPOSITION};
use reqwest::Url;
use std::fs::File;
use std::io;
use std::path::Path;
#[cfg(windows)]
use std::path::{Component, Prefix, PathBuf};

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

#[cfg(windows)]
fn get_path_prefix(path: &Path) -> Prefix {
    match path.components().next().unwrap() {
        Component::Prefix(prefix_component) => prefix_component.kind(),
        _ => panic!(),
    }
}

#[cfg(windows)]
pub fn prepend_long_path_support<P:AsRef<Path>>(path:P) -> PathBuf {
    use std::ffi::OsString;

    let path = path.as_ref();
    if (path.has_root() && !path.is_absolute()) || (path.is_absolute() && !get_path_prefix(path).is_verbatim()) {
        trace!(r#"prepend path with \\?\"#);
        let mut components = path.components();
        let mut new_prefix = OsString::new();
        let mut new_path = PathBuf::new();

        new_prefix.push(r"\\?\");
        new_prefix.push(components.next().unwrap());

        new_path.push(new_prefix);
        while let Some(component) = components.next() {
            new_path.push(component);
        }
        new_path
    } else {
        path.to_path_buf()
    }
}

pub struct UrlUtils {}

impl UrlUtils {
    fn get_final_file_name_from_url(url: &Url) -> io::Result<String> {
        let client = Client::new();
        let response = client
            .head(url.clone())
            .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.100 Safari/537.36")
            .send()
            .map_err(|err| {
                io::Error::new(io::ErrorKind::Other, err)
            })?;

        response
            .headers()
            .get(CONTENT_DISPOSITION)
            .and_then(|disposition| {
                if disposition.is_empty() {
                    None
                } else {
                    Some(disposition)
                }
            })
            .and_then(|disposition| {
                let disposition = disposition.to_str().ok()?;
                trace!("disposition header value: {}", disposition);
                let parts = disposition.split(';');
                parts
                    .map(|s| s.trim())
                    .fold(None, {
                        |filename: Option<String>, part| {
                            if part.starts_with("filename=") {
                                let part = part.replace("filename=", "");
                                let part = &part.trim_start_matches('"').trim_end_matches('"');
                                Some(part.to_string())
                            } else {
                                filename
                            }
                        }
                    })
                    .map(|name| {
                        trace!("after header disposition replacement");
                        trace!("{}", &name);
                        name
                    })
            })
            .or_else(|| {
                response
                    .url()
                    .as_str()
                    .rsplit('/')
                    .next()
                    .map(|s| s.to_string())
            })
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "unable to parse final filename")
            })
    }

    pub fn get_file_name_from_url(url: &Url) -> io::Result<String> {
        let test_path = Path::new(url.as_ref());
        if test_path.extension().is_some() {
            url.as_str()
                .rsplit('/')
                .next()
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("unable to read filename from url {}", url),
                    )
                })
        } else {
            Self::get_final_file_name_from_url(url)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Url;

    #[test]
    fn parse_file_name_from_url_with_file_name_part() {
        let url = Url::parse("https://beta.unity3d.com/download/8ea4afdbfa47/MacEditorTargetInstaller/UnitySetup-Android-Support-for-Editor-2019.3.0a8.pkg").unwrap();
        assert_eq!(UrlUtils::get_file_name_from_url(&url).unwrap(), "UnitySetup-Android-Support-for-Editor-2019.3.0a8.pkg".to_string());
    }

    #[test]
    fn parse_file_name_from_url_without_file_name_part_and_content_disposition() {
        let url = Url::parse("https://go.microsoft.com/fwlink/?linkid=2086937").unwrap();
        assert!(UrlUtils::get_file_name_from_url(&url).unwrap().starts_with("visualstudioformac-"));
    }

    #[test]
    fn parse_file_name_from_url_without_file_name_part_and_content_disposition2() {
        let url = Url::parse("https://go.microsoft.com/fwlink/?linkid=2087047").unwrap();
        assert!(UrlUtils::get_file_name_from_url(&url).unwrap().starts_with("monoframework-mdk-"));
    }

    #[test]
    fn parse_file_name_from_url_without_file_name_part_and_content_disposition3() {
        let url = Url::parse("https://new-translate.unity3d.jp/v1/live/54/2019.3/zh-hant").unwrap();
        assert_eq!(UrlUtils::get_file_name_from_url(&url).unwrap(), "zh-hant.po".to_string());
    }

    #[cfg(windows)]
    #[test]
    fn prepend_long_path_prefix_when_missing() {
        let path = Path::new(r#"c:/path/to/some/file.txt"#);
        let new_path = prepend_long_path_support(&path);
        assert!(new_path.to_string_lossy().starts_with(r#"\\?\c:\"#));
    }

    #[cfg(windows)]
    #[test]
    fn prepend_long_path_prefix_when_missing2() {
        let path = Path::new(r#"/path/to/some/file.txt"#);
        let new_path = prepend_long_path_support(&path);
        assert!(new_path.to_string_lossy().starts_with(r#"\\?\"#));
    }

    #[cfg(windows)]
    #[test]
    fn prepend_long_path_changes_path_separator() {
        let path = Path::new(r#"c:/path/to/some/file.txt"#);
        let new_path = prepend_long_path_support(&path);
        assert_eq!(new_path.to_string_lossy() , r#"\\?\c:\path\to\some\file.txt"#);
    }

    #[cfg(windows)]
    #[test]
    fn prepend_long_path_prefix_only_absolute_paths() {
        let path = Path::new(r#"./some/file.txt"#);
        let new_path = prepend_long_path_support(&path);
        assert!(!new_path.to_string_lossy().starts_with(r#"\\?\"#));
    }

    #[cfg(windows)]
    #[test]
    fn prepend_long_path_prefix_returns_same_path_when_already_prefixed() {
        let path = Path::new(r#"\\?\c:/path/to/some/file.txt"#);
        let new_path = prepend_long_path_support(&path);
        assert_eq!(path.to_str(), new_path.to_str());
    }
}
