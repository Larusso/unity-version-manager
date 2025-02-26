use serde::{Deserialize, Serialize};

use crate::error::ListVersionsError;
use crate::{UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseStream};

#[derive(Debug)]
pub struct ListVersions(std::vec::IntoIter<String>);

impl Iterator for ListVersions {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl ListVersions {
    pub fn builder() -> ListVersionsBuilder {
        ListVersionsBuilder::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ListVersionsBuilder {
    architecture: UnityReleaseDownloadArchitecture,
    platform: UnityReleaseDownloadPlatform,
    skip: usize,
    limit: usize,
    stream: UnityReleaseStream,
    include_revision: bool,
    autopage: bool,
}

impl ListVersionsBuilder {
    fn new() -> Self {
        Self {
            architecture: Default::default(),
            platform: Default::default(),
            skip: Default::default(),
            limit: 100,
            stream: Default::default(),
            include_revision: false,
            autopage: false,
        }
    }

    pub fn list(self) -> Result<ListVersions, ListVersionsError> {
        let mut result = vec![];
        let mut p = self.send()?;
    
        loop {
            result.append(&mut p.content);
            if self.autopage && p.has_next_page() {
                p = p.next_page().expect("next page")?;
            } else {
                break;
            }
        }
    
        Ok(ListVersions(result.into_iter())) 
    } 

    fn send(self) -> Result<ListVersionsPageResult, ListVersionsError> {
        let url = "https://live-platform-api.prd.ld.unity3d.com/graphql";
        let request_body = ListVersionsRequestBody::new(self.into());
        let client = reqwest::blocking::Client::new();
        let res: ListVersionsResultBody = client
            .post(url)
            .json(&request_body)
            .send()
            .map_err(ListVersionsError::NetworkError)?
            .json()
            .map_err(ListVersionsError::JsonError)?;
        let page_info = res.data.get_unity_releases.page_info;
        let versions = res.data.get_unity_releases.edges.iter().map(|e| {
            if self.include_revision {
                format!("{} ({})", e.node.version, e.node.short_revision)
            } else {
                e.node.version.to_string()
            }
        });

        let next_page_call = if page_info.has_next_page {
            let mut b = self.clone();
            b = self.skip(b.skip.checked_add(b.limit).unwrap_or(usize::MAX));
            Some(b)
        } else {
            None
        };

        let r = ListVersionsPageResult::new(versions, next_page_call);
        Ok(r)
    }

    pub fn architecture(mut self, architecture: UnityReleaseDownloadArchitecture) -> Self {
        self.architecture = architecture;
        self
    }

    pub fn platform(mut self, platform: UnityReleaseDownloadPlatform) -> Self {
        self.platform = platform;
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn stream(mut self, stream: UnityReleaseStream) -> Self {
        self.stream = stream;
        self
    }

    pub fn include_revision(mut self, include_revision: bool) -> Self {
        self.include_revision = include_revision;
        self
    }

    pub fn autopage(mut self, autopage: bool) -> Self {
        self.autopage = autopage;
        self
    }
}



const LIST_VERSIONS_QUERY: &str = include_str!("list_versions_query.graphql");

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListVersionsOptions {
    architecture: UnityReleaseDownloadArchitecture,
    platform: UnityReleaseDownloadPlatform,
    skip: usize,
    limit: usize,
    stream: UnityReleaseStream,
}

impl Default for ListVersionsOptions {
    fn default() -> Self {
        Self {
            architecture: Default::default(),
            platform: Default::default(),
            skip: Default::default(),
            limit: 100,
            stream: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListVersionsRequestBody {
    query: String,
    variables: ListVersionsOptions,
}

impl ListVersionsRequestBody {
    pub fn new(variables: ListVersionsOptions) -> Self {
        Self {
            query: LIST_VERSIONS_QUERY.to_string(),
            variables,
        }
    }
}

impl Default for ListVersionsRequestBody {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListVersionsResultBody {
    data: ListVersionsResultBodyData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListVersionsResultBodyData {
    get_unity_releases: GetUnityReleases,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetUnityReleases {
    edges: Vec<UnityReleaseOffsetEdge>,
    page_info: PageInfo,
    total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityReleaseOffsetEdge {
    node: UnityRelease,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityRelease {
    version: String,
    short_revision: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
}



#[derive(Debug)]
struct ListVersionsPageResult {
    next_page_options: Option<ListVersionsBuilder>,
    content: Vec<String>,
}

impl ListVersionsPageResult {
    pub fn new(
        content: impl IntoIterator<Item = String>,
        next_page_options: Option<ListVersionsBuilder>,
    ) -> Self {
        Self {
            next_page_options,
            content: content.into_iter().collect(),
        }
    }

    pub fn has_next_page(&self) -> bool {
        self.next_page_options.is_some()
    }

    pub fn next_page(self) -> Option<Result<Self, ListVersionsError>> {
        let b = self.next_page_options?;
        Some(b.send())
    }
}

impl IntoIterator for ListVersionsPageResult {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}

impl From<ListVersionsBuilder> for ListVersionsOptions {
    fn from(value: ListVersionsBuilder) -> Self {
        Self {
            architecture: value.architecture,
            platform: value.platform,
            skip: value.skip,
            limit: value.limit,
            stream: value.stream,
        }
    }
}