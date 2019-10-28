#[cfg(unix)]
use cluFlock::{ExclusiveFlock, FlockLock};
use reqwest::Client;
use reqwest::header::{USER_AGENT, CONTENT_DISPOSITION};
use reqwest::Url;
use std::fs::File;
use std::io;
use std::path::Path;

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
        assert_eq!(UrlUtils::get_file_name_from_url(&url).unwrap(), "visualstudioformac-8.3.4.8.dmg".to_string());
    }

    #[test]
    fn parse_file_name_from_url_without_file_name_part_and_content_disposition2() {
        let url = Url::parse("https://go.microsoft.com/fwlink/?linkid=2087047").unwrap();
        assert_eq!(UrlUtils::get_file_name_from_url(&url).unwrap(), "monoframework-mdk-6.4.0.208.macos10.xamarin.universal.pkg".to_string());
    }

    #[test]
    fn parse_file_name_from_url_without_file_name_part_and_content_disposition3() {
        let url = Url::parse("https://new-translate.unity3d.jp/v1/live/54/2019.3/zh-hant").unwrap();
        assert_eq!(UrlUtils::get_file_name_from_url(&url).unwrap(), "zh-hant.po".to_string());
    }
}
