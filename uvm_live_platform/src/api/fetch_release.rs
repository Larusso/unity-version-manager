use crate::error::FetchReleaseError;
use crate::{Release, UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseEntitlement, UnityReleaseStream};
use crate::api::middleware::MiddlewareChain;
#[cfg(feature = "cache")]
use crate::api::cache::{CacheMiddleware, CacheConfig};
use serde::{Deserialize, Serialize};
use unity_version::Version;

#[cfg(feature = "cache")]
// Internal type alias for cache middleware - not exposed publicly  
type FetchReleaseCacheMiddleware = CacheMiddleware<FetchReleaseOptions, Release>;
// Internal type alias for middleware chain - used multiple times in this module
type FetchReleaseMiddlewareChain<'a> = MiddlewareChain<'a, FetchReleaseOptions, Release, FetchReleaseError>;

#[cfg(feature = "cache")]
impl FetchReleaseCacheMiddleware {
    /// Create cache middleware with configuration from environment variables
    /// Looks for UVM_LIVE_PLATFORM_FETCH_RELEASE_CACHE_* variables first,
    /// then falls back to UVM_LIVE_PLATFORM_CACHE_* variables
    pub fn from_env() -> Self {
        // FetchRelease data is very stable - cache for 7 days by default
        let config = CacheConfig::from_env_with_prefix_and_default(
            Some("FETCH_RELEASE"), 
            Some(7 * 24 * 60 * 60) // 7 days
        );
        Self::new(config)
    }
}

#[derive(Debug)]
pub struct FetchRelease {}

impl FetchRelease {
    pub fn builder<V: Into<Version>>(version: V) -> FetchReleaseBuilder<'static> {
        FetchReleaseBuilder::new(version.into())
    }
    
    pub fn try_builder<V: TryInto<Version>>(version: V) -> Result<FetchReleaseBuilder<'static>, V::Error> {
        version.try_into().map(|v| FetchReleaseBuilder::new(v))
    }
}

#[derive(Debug)]
pub struct FetchReleaseBuilder<'a> {
    architecture: Vec<UnityReleaseDownloadArchitecture>,
    platform: Vec<UnityReleaseDownloadPlatform>,
    stream: Vec<UnityReleaseStream>,
    entitlements: Vec<UnityReleaseEntitlement>,
    version: Version,
    middleware: FetchReleaseMiddlewareChain<'a>,
}

impl<'a> FetchReleaseBuilder<'a> {
    fn new(version: Version) -> Self {
        Self {
            version,
            architecture: Default::default(),
            platform: Default::default(),
            stream: Default::default(),
            entitlements: Default::default(),
            middleware: {
                #[cfg(feature = "cache")]
                {
                    FetchReleaseMiddlewareChain::new().add(FetchReleaseCacheMiddleware::from_env())
                }
                #[cfg(not(feature = "cache"))]
                {
                    FetchReleaseMiddlewareChain::new()
                }
            },
        }
    }

    /// Control caching for this request
    pub fn without_cache(mut self, no_cache: bool) -> Self {
        if no_cache {
            self.middleware = FetchReleaseMiddlewareChain::new();
        }
        self
    }
    
    #[cfg(feature = "cache")]
    /// Enable refresh mode - always fetch fresh data but still cache it
    /// Useful for --refresh flags
    pub fn with_refresh(mut self, refresh: bool) -> Self {
        if refresh {
            self.middleware = FetchReleaseMiddlewareChain::new().add(FetchReleaseCacheMiddleware::refresh_mode());
        }
        self
    }

    pub fn fetch(self) -> Result<Release, FetchReleaseError> {
        self.send()
    }

    pub fn send(self) -> Result<Release, FetchReleaseError> {
        let version = self.version.clone();
        let architecture = self.architecture.clone();
        let stream = self.stream.clone();
        let platform = self.platform.clone();
        
        let fetch_options = FetchReleaseOptions {
            architecture: self.architecture.clone(),
            platform: self.platform.clone(),
            stream: self.stream.clone(),
            version: self.version.to_string(),
            entitlements: self.entitlements.clone(),
        };
        
        // Define the core fetch logic that will be called by middlewares
        let core_fetch = |options: &FetchReleaseOptions| -> Result<Release, FetchReleaseError> {
            let url = "https://live-platform-api.prd.ld.unity3d.com/graphql";
            let request_body = FetchReleaseRequestBody::new(options.clone());
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
        };

        // Execute the middleware chain with the core fetch logic
        let middleware = self.middleware;
        middleware.execute(&fetch_options, core_fetch)
    }

    pub fn with_system_architecture(self) -> Self {
        self.with_architecture(Default::default())
    }

    pub fn with_architecture(mut self, architecture: UnityReleaseDownloadArchitecture) -> Self {
        self.architecture.push(architecture);
        self
    }

    pub fn with_current_platform(self) -> Self {
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

    pub fn for_current_system(self) -> Self {
        self.with_system_architecture()
            .with_current_platform()
    }

}

const FETCH_RELEASE_QUERY: &str = include_str!("fetch_release_query.graphql");

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchReleaseOptions {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub architecture: Vec<UnityReleaseDownloadArchitecture>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub platform: Vec<UnityReleaseDownloadPlatform>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stream: Vec<UnityReleaseStream>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entitlements: Vec<UnityReleaseEntitlement>,
    pub version: String,
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

impl<'a> From<FetchReleaseBuilder<'a>> for FetchReleaseOptions {
    fn from(value: FetchReleaseBuilder<'a>) -> Self {
        Self {
            architecture: value.architecture,
            platform: value.platform,
            stream: value.stream,
            version: value.version.to_string(),
            entitlements: value.entitlements,
        }
    }
}
