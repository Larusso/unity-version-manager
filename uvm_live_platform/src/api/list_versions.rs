use serde::{Deserialize, Serialize};

use crate::api::middleware::MiddlewareChain;
use crate::error::ListVersionsError;
use crate::{UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseEntitlement, UnityReleaseStream};
#[cfg(feature = "cache")]
use crate::api::cache::{CacheMiddleware, CacheConfig};

#[cfg(feature = "cache")]
// Internal type alias for cache middleware - not exposed publicly
type ListVersionsPageCacheMiddleware = CacheMiddleware<ListVersionsOptions, ListVersionsPageResult>;
// Internal type alias for middleware chain - used multiple times in this module
type ListVersionsMiddlewareChain<'a> = MiddlewareChain<'a, ListVersionsOptions, ListVersionsPageResult, ListVersionsError>;

#[cfg(feature = "cache")]
impl ListVersionsPageCacheMiddleware {
    pub fn from_env() -> Self {
        // ListVersions data changes more frequently - cache for 2 hours by default
        let config = CacheConfig::from_env_with_prefix_and_default(
            Some("LIST_VERSIONS"), 
            Some(2 * 60 * 60) // 2 hours
        );
        Self::new(config)
    }
}

#[derive(Debug)]
pub struct ListVersions(std::vec::IntoIter<String>);

impl Iterator for ListVersions {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl ListVersions {
    pub fn builder() -> ListVersionsBuilder<'static> {
        ListVersionsBuilder::new()
    }
}

#[derive(Debug)]
pub struct ListVersionsBuilder<'a> {
    architecture: Vec<UnityReleaseDownloadArchitecture>,
    platform: Vec<UnityReleaseDownloadPlatform>,
    skip: usize,
    limit: usize,
    stream: Vec<UnityReleaseStream>,
    entitlements: Vec<UnityReleaseEntitlement>,
    include_revision: bool,
    autopage: bool,
    version: Option<String>,
    middleware: ListVersionsMiddlewareChain<'a>,
}

impl<'a> ListVersionsBuilder<'a> {
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
            middleware: {
                #[cfg(feature = "cache")]
                {
                    ListVersionsMiddlewareChain::new().add(ListVersionsPageCacheMiddleware::from_env())
                }
                #[cfg(not(feature = "cache"))]
                {
                    ListVersionsMiddlewareChain::new()
                }
            },
        }
    }

    pub fn without_cache(mut self, no_cache: bool) -> Self {
        if no_cache {
            self.middleware = ListVersionsMiddlewareChain::new();
        }
        self
    }
    
    #[cfg(feature = "cache")]
    /// Enable refresh mode - always fetch fresh data but still cache it
    /// Useful for --refresh flags
    pub fn with_refresh(mut self, refresh: bool) -> Self {
        if refresh {
            self.middleware = ListVersionsMiddlewareChain::new().add(ListVersionsPageCacheMiddleware::refresh_mode());
        }
        self
    }

    pub fn list(self) -> Result<ListVersions, ListVersionsError> {
        let mut result = vec![];
        let autopage = self.autopage;
        let mut page = self.send()?;  // Use send() which includes middleware/caching!
        
        loop {
            result.append(&mut page.content);
            if autopage && page.has_next_page() {
                page = page.next_page().expect("next page")?;  // This also uses middleware!
            } else {
                break;
            }
        }
        
        Ok(ListVersions(result.into_iter())) 
    } 

    pub fn send(self) -> Result<ListVersionsPageResult, ListVersionsError> {
        // Extract values we need before moving self
        let include_revision = self.include_revision;

        let list_options = ListVersionsOptions {
            architecture: self.architecture.clone(),
            platform: self.platform.clone(),
            skip: self.skip,
            limit: self.limit,
            stream: self.stream.clone(),
            entitlements: self.entitlements.clone(),
            version: self.version.clone(),
        };

        // Define the core fetch logic that will be called by middlewares
        let core_fetch = move |options: &ListVersionsOptions| -> Result<ListVersionsPageResult, ListVersionsError> {
            let url = "https://live-platform-api.prd.ld.unity3d.com/graphql";
            let request_body = ListVersionsRequestBody::new(options.clone());
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
            
            let result = ListVersionsPageResult::new(
                versions,
                page_info.has_next_page,
                options.skip,        // Use options instead of captured variables
                options.limit,       // Use options instead of captured variables
                options.architecture.clone(),
                options.platform.clone(),
                options.stream.clone(),
                options.entitlements.clone(),
                options.version.clone(),
                include_revision,
            );
            Ok(result)
        };

        // Execute the middleware chain with the core fetch logic
        let middleware = self.middleware;
        middleware.execute(&list_options, core_fetch)
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

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
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



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVersionsPageResult {
    pub content: Vec<String>,
    // Store pagination state that can be serialized
    has_next_page: bool,
    current_skip: usize,
    current_limit: usize,
    // Store the original request parameters to reconstruct next page
    architecture: Vec<UnityReleaseDownloadArchitecture>,
    platform: Vec<UnityReleaseDownloadPlatform>,
    stream: Vec<UnityReleaseStream>,
    entitlements: Vec<UnityReleaseEntitlement>,
    version: Option<String>,
    include_revision: bool,
}

impl ListVersionsPageResult {
    pub fn new(
        content: impl IntoIterator<Item = String>,
        has_next_page: bool,
        current_skip: usize,
        current_limit: usize,
        architecture: Vec<UnityReleaseDownloadArchitecture>,
        platform: Vec<UnityReleaseDownloadPlatform>,
        stream: Vec<UnityReleaseStream>,
        entitlements: Vec<UnityReleaseEntitlement>,
        version: Option<String>,
        include_revision: bool,
    ) -> Self {
        Self {
            content: content.into_iter().collect(),
            has_next_page,
            current_skip,
            current_limit,
            architecture,
            platform,
            stream,
            entitlements,
            version,
            include_revision,
        }
    }

    pub fn has_next_page(&self) -> bool {
        self.has_next_page
    }

    pub fn next_page(self) -> Option<Result<Self, ListVersionsError>> {
        if !self.has_next_page {
            return None;
        }

        // Reconstruct the builder for the next page
        let next_skip = self.current_skip.checked_add(self.current_limit)?;
        
        let builder = ListVersionsBuilder::new()
            .skip(next_skip)
            .limit(self.current_limit)
            .with_architectures(self.architecture)
            .with_platforms(self.platform)
            .with_streams(self.stream)
            .with_entitlements(self.entitlements)
            .include_revision(self.include_revision);
            
        let builder = if let Some(version) = self.version {
            builder.with_version(version)
        } else {
            builder
        };

        Some(builder.send())
    }
}

impl IntoIterator for ListVersionsPageResult {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}

impl<'a> From<ListVersionsBuilder<'a>> for ListVersionsOptions {
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