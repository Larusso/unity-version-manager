use serde::{Deserialize, Serialize};

use crate::error::ListVersionsError;
use crate::{UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseEntitlement, UnityReleaseStream};

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

#[derive(Debug, Clone)]
pub struct ListVersionsBuilder {
    architecture: Vec<UnityReleaseDownloadArchitecture>,
    platform: Vec<UnityReleaseDownloadPlatform>,
    skip: usize,
    limit: usize,
    stream: Vec<UnityReleaseStream>,
    entitlements: Vec<UnityReleaseEntitlement>,
    include_revision: bool,
    autopage: bool,
    version: Option<String>,
}

impl ListVersionsBuilder {
    fn new() -> Self {
        Self {
            architecture: Default::default(),
            platform: Default::default(),
            skip: Default::default(),
            limit: 100,
            stream: Default::default(),
            entitlements: Default::default(),
            include_revision: false,
            autopage: false,
            version: None,
        }
    }

    pub fn list(self) -> Result<ListVersions, ListVersionsError> {
        let mut result = vec![];
        let autopage = self.autopage;
        let mut p = self.send()?;
        loop {
            result.append(&mut p.content);
            if autopage && p.has_next_page() {
                p = p.next_page().expect("next page")?;
            } else {
                break;
            }
        }
    
        Ok(ListVersions(result.into_iter())) 
    } 

    fn send(self) -> Result<ListVersionsPageResult, ListVersionsError> {
        let url = "https://live-platform-api.prd.ld.unity3d.com/graphql";
        let version_builder_next_page = self.clone();
        let include_revision = self.include_revision;
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
            if include_revision {
                format!("{} ({})", e.node.version, e.node.short_revision)
            } else {
                e.node.version.to_string()
            }
        });

        let next_page_call = if page_info.has_next_page {
            let mut b = version_builder_next_page.clone();
            let skip = b.skip;
            let limit = b.limit;
            b = b.skip(skip.checked_add(limit).unwrap_or(usize::MAX));
            Some(b)
        } else {
            None
        };

        let r = ListVersionsPageResult::new(versions, next_page_call);
        Ok(r)
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
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

    pub fn with_system_architecture(self) -> Self {
        self.with_architecture(Default::default())
    }

    pub fn with_architecture(mut self, architecture: UnityReleaseDownloadArchitecture) -> Self {
        self.architecture.push(architecture);
        self
    }

    pub fn with_architectures<I: IntoIterator<Item = UnityReleaseDownloadArchitecture> >(mut self, architectures: I) -> Self {
        self.architecture.extend(architectures);
        self
    }

    pub fn with_current_platform(self) -> Self {
        self.with_platform(Default::default())
    }

    pub fn with_platform(mut self, platform: UnityReleaseDownloadPlatform) -> Self {
        self.platform.push(platform);
        self
    }

    pub fn with_platforms<I: IntoIterator<Item = UnityReleaseDownloadPlatform> >(mut self, platforms: I) -> Self {
        self.platform.extend(platforms);
        self
    }

    pub fn for_current_system(self) -> Self {
        self.with_system_architecture()
            .with_current_platform()
    }

    pub fn with_stream(mut self, stream: UnityReleaseStream) -> Self {
        self.stream.push(stream);
        self
    }

    pub fn with_streams<I: IntoIterator<Item = UnityReleaseStream> >(mut self, streams: I) -> Self {
        self.stream.extend(streams);
        self
    }

    pub fn with_extended_lts(self) -> Self {
        self.with_entitlement(UnityReleaseEntitlement::Xlts)
    }

    pub fn with_u7_alpha(self) -> Self {
        self.with_entitlement(UnityReleaseEntitlement::U7Alpha)
    }

    pub fn with_entitlement(mut self, entitlement: UnityReleaseEntitlement) -> Self {
        self.entitlements.push(entitlement);
        self
    }

    pub fn with_entitlements<I: IntoIterator<Item = UnityReleaseEntitlement> >(mut self, entitlements: I) -> Self {
        self.entitlements.extend(entitlements);
        self
    }

    pub fn with_version<S:Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }
}



const LIST_VERSIONS_QUERY: &str = include_str!("list_versions_query.graphql");

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListVersionsOptions {
    architecture: Vec<UnityReleaseDownloadArchitecture>,
    platform: Vec<UnityReleaseDownloadPlatform>,
    skip: usize,
    limit: usize,
    stream: Vec<UnityReleaseStream>,
    entitlements: Vec<UnityReleaseEntitlement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

impl Default for ListVersionsOptions {
    fn default() -> Self {
        Self {
            architecture: Default::default(),
            platform: Default::default(),
            skip: Default::default(),
            limit: 100,
            stream: Default::default(),
            entitlements: Default::default(),
            version: None,
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
            entitlements: value.entitlements,
            version: value.version,
        }
    }
}