use crate::error::FetchReleaseError;
use crate::{
    Release, UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseStream,
};
use serde::{Deserialize, Serialize};
use unity_version::Version;

#[derive(Debug)]
pub struct FetchRelease {}

impl FetchRelease {
    pub fn builder<V: Into<Version>>(version: V) -> FetchReleaseBuilder {
        FetchReleaseBuilder::new(version.into())
    }
    pub fn try_builder<V: TryInto<Version>>(version: V) -> Result<FetchReleaseBuilder, V::Error> {
        version.try_into().map(|v| FetchReleaseBuilder::new(v))
    }
}

#[derive(Debug, Clone)]
pub struct FetchReleaseBuilder {
    architecture: Vec<UnityReleaseDownloadArchitecture>,
    platform: Vec<UnityReleaseDownloadPlatform>,
    stream: Vec<UnityReleaseStream>,
    entitlements: Vec<UnityReleaseEntitlement>,
    version: Version,
}

impl FetchReleaseBuilder {
    fn new(version: Version) -> Self {
        Self {
            version,
            architecture: Default::default(),
            platform: Default::default(),
            stream: Default::default(),
            entitlements: Default::default(),
        }
    }

    pub fn fetch(self) -> Result<Release, FetchReleaseError> {
        self.send()
    }

    pub fn send(self) -> Result<Release, FetchReleaseError> {
        let version = self.version.clone();
        let architecture = self.architecture.clone();
        let stream = self.stream.clone();
        let platform = self.platform.clone();
        let url = "https://live-platform-api.prd.ld.unity3d.com/graphql";
        let request_body = FetchReleaseRequestBody::new(self.into());
        let client = reqwest::blocking::Client::new();
        let mut res: FetchReleaseResultBody = client
            .post(url)
            .json(&request_body)
            .send()
            .map_err(FetchReleaseError::NetworkError)?
            .json()
            .map_err(|err| FetchReleaseError::JsonError(err))?;

        if res.data.get_unity_releases.edges.is_empty() {
            let p = platform.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",");
            let a = architecture.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",");
            let s = stream.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(",");
            Err(FetchReleaseError::NotFound(version.to_string(), p, a, s))
        } else {
            let release_edge = res.data.get_unity_releases.edges.remove(0);
            let release = release_edge.node;
            Ok(release)
        }
    }

    pub fn with_system_architecture(mut self) -> Self {
        self.with_architecture(Default::default())
    }

    pub fn with_architecture(mut self, architecture: UnityReleaseDownloadArchitecture) -> Self {
        self.architecture.push(architecture);
        self
    }

    pub fn with_current_platform(mut self) -> Self {
        self.with_platform(Default::default())
    }

    pub fn with_platform(mut self, platform: UnityReleaseDownloadPlatform) -> Self {
        self.platform.push(platform);
        self
    }

    pub fn with_stream(mut self, stream: UnityReleaseStream) -> Self {
        self.stream.push(stream);
        self
    }

    pub fn with_extended_lts(mut self) -> Self {
        self.with_entitlement(UnityReleaseEntitlement::Xlts)
    }

    pub fn with_u7_alpha(mut self) -> Self {
        self.with_entitlement(UnityReleaseEntitlement::U7Alpha)
    }

    pub fn with_entitlement(mut self, entitlement: UnityReleaseEntitlement) -> Self {
        self.entitlements.push(entitlement);
        self
    }

    pub fn for_current_system(mut self) -> Self {
        self.with_system_architecture()
            .with_current_platform()
    }
}

const FETCH_RELEASE_QUERY: &str = include_str!("fetch_release_query.graphql");

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchReleaseOptions {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    architecture: Vec<UnityReleaseDownloadArchitecture>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    platform: Vec<UnityReleaseDownloadPlatform>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stream: Vec<UnityReleaseStream>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    entitlements: Vec<UnityReleaseEntitlement>,
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
            version: value.version.to_string(),
            entitlements: value.entitlements,
        }
    }
}
