use serde::{Serialize, Deserialize};

use crate::{UnityReleaseStream, UnityReleaseDownloadPlatform, UnityReleaseDownloadArchitecture, Result, error::LivePlatformError};

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

    pub fn list(self) -> Result<ListVersions> {
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

    fn send(self) -> Result<ListVersionsPageResult> {
        let url = "https://live-platform-api.prd.ld.unity3d.com/graphql";
        let request_body = UnityReleaseDownloadGetUnityReleasesRequestBody::new(self.into());
        let client = reqwest::blocking::Client::new();
        let res: UnityReleaseDownloadGetUnityReleasesResultBody = client
            .post(url)
            .json(&request_body)
            .send()
            .map_err(|err| LivePlatformError {
                msg: "Call to LivePlatform service failed".to_string(),
                source: err.into(),
            })?
            .json()
            .map_err(|err| LivePlatformError {
                msg: "Json serialization failed".to_string(),
                source: err.into(),
            })?;
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
struct UnityReleaseDownloadGetUnityReleasesOptions {
    architecture: UnityReleaseDownloadArchitecture,
    platform: UnityReleaseDownloadPlatform,
    skip: usize,
    limit: usize,
    stream: UnityReleaseStream,
}

impl Default for UnityReleaseDownloadGetUnityReleasesOptions {
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
struct UnityReleaseDownloadGetUnityReleasesRequestBody {
    query: String,
    variables: UnityReleaseDownloadGetUnityReleasesOptions,
}

impl UnityReleaseDownloadGetUnityReleasesRequestBody {
    pub fn new(variables: UnityReleaseDownloadGetUnityReleasesOptions) -> Self {
        Self {
            query: LIST_VERSIONS_QUERY.to_string(),
            variables: variables,
        }
    }
}

impl Default for UnityReleaseDownloadGetUnityReleasesRequestBody {
    fn default() -> Self {
        Self {
            query: LIST_VERSIONS_QUERY.to_string(),
            variables: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityReleaseDownloadGetUnityReleasesResultBody {
    data: UnityReleaseDownloadGetUnityReleasesResultBodyData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityReleaseDownloadGetUnityReleasesResultBodyData {
    get_unity_releases: UnityReleaseDownloadGetUnityReleasesResultBodyDataGetUnityReleases,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityReleaseDownloadGetUnityReleasesResultBodyDataGetUnityReleases {
    edges: Vec<UnityReleaseDownloadGetUnityReleasesResultBodyDataGetUnityReleasesEdge>,
    page_info: PageInfo,
    total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityReleaseDownloadGetUnityReleasesResultBodyDataGetUnityReleasesEdge {
    node: UnityReleaseDownloadGetUnityReleasesResultBodyDataGetUnityReleasesEdgeNode,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityReleaseDownloadGetUnityReleasesResultBodyDataGetUnityReleasesEdgeNode {
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
            next_page_options: next_page_options,
            content: content.into_iter().collect(),
        }
    }

    pub fn has_next_page(&self) -> bool {
        self.next_page_options.is_some()
    }

    pub fn next_page(self) -> Option<Result<Self>> {
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



impl From<ListVersionsBuilder> for UnityReleaseDownloadGetUnityReleasesOptions {
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