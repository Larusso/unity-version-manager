use reqwest::header;
use reqwest::Url;
use std::time::Duration;

lazy_static! {
    pub static ref CLIENT: reqwest::Client = {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("uvm"));
        reqwest::Client::builder()
            .gzip(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Create http client")
    };
}
