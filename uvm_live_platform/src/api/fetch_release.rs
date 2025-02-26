use crate::error::FetchReleaseError;
use crate::{Release, UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseStream};
use serde::{Deserialize, Serialize};
use unity_version::Version;

#[derive(Debug)]
pub struct FetchRelease {}

impl FetchRelease {
    pub fn builder<V: Into<Version>>(version: V) -> FetchReleaseBuilder { FetchReleaseBuilder::new(version.into().to_string()) }
}


#[derive(Debug, Clone)]
pub struct FetchReleaseBuilder {
    architecture: UnityReleaseDownloadArchitecture,
    platform: UnityReleaseDownloadPlatform,
    stream: UnityReleaseStream,
    version: String
}

impl FetchReleaseBuilder {
    fn new(version: String) -> Self {
        Self {
            version,
            architecture: Default::default(),
            platform: Default::default(),
            stream: Default::default(),
        }
    }

    pub fn fetch(self) -> Result<Release, FetchReleaseError> {
        self.send()
    }

    pub fn send(self) -> Result<Release, FetchReleaseError> {
        let version = self.version.clone();
        let url = "https://live-platform-api.prd.ld.unity3d.com/graphql";
        let request_body = FetchReleaseRequestBody::new(self.into());
        let client = reqwest::blocking::Client::new();
        let mut res: FetchReleaseResultBody = client
            .post(url)
            .json(&request_body)
            .send()
            .map_err(FetchReleaseError::NetworkError)?
            .json()
            .map_err(FetchReleaseError::JsonError)?;

        if res.data.get_unity_releases.edges.is_empty() {
            Err(FetchReleaseError::NotFound(version))
        } else {
            let release_edge = res.data.get_unity_releases.edges.remove(0);
            let release = release_edge.node;
            Ok(release)
        }
    }

    pub fn architecture(mut self, architecture: UnityReleaseDownloadArchitecture) -> Self {
        self.architecture = architecture;
        self
    }

    pub fn platform(mut self, platform: UnityReleaseDownloadPlatform) -> Self {
        self.platform = platform;
        self
    }

    pub fn stream(mut self, stream: UnityReleaseStream) -> Self {
        self.stream = stream;
        self
    }
}

const FETCH_RELEASE_QUERY: &str = include_str!("fetch_release_query.graphql");

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchReleaseOptions {
    architecture: UnityReleaseDownloadArchitecture,
    platform: UnityReleaseDownloadPlatform,
    stream: UnityReleaseStream,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchReleaseRequestBody {
    query: String,
    variables: FetchReleaseOptions,
}

impl FetchReleaseRequestBody {
    pub fn new(variables: FetchReleaseOptions) -> Self {
        Self {
            query: FETCH_RELEASE_QUERY.to_string(),
            variables,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchReleaseResultBody {
    data: FetchReleaseResultBodyData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchReleaseResultBodyData {
    get_unity_releases: GetUnityReleases,
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
pub enum UnityReleaseOrder {
    #[serde(rename(serialize = "RELEASE_DATE_ASC"))]
    ReleaseDateAscending,
    #[serde(rename(serialize = "RELEASE_DATE_DESC"))]
    ReleaseDateDescending,
}

impl Default for UnityReleaseOrder {
    fn default() -> Self {
        Self::ReleaseDateDescending
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnityReleaseEntitlement {
    Xlts,
    U7Alpha,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetUnityReleases {
    edges: Vec<UnityReleaseOffsetEdge>,
    total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnityReleaseOffsetEdge {
    node: Release,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
}

impl From<FetchReleaseBuilder> for FetchReleaseOptions {
    fn from(value: FetchReleaseBuilder) -> Self {
        Self {
            architecture: value.architecture,
            platform: value.platform,
            stream: value.stream,
            version: value.version,
        }
    }
}