//! Cache functionality for the uvm_live_platform crate
//! 
//! This module provides a generic caching system with middleware integration.
//! All functionality in this module is only available when the `cache` feature is enabled.

use crate::api::middleware::Middleware;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use dirs_2;
use thiserror::Error;

/// Errors that can occur during cache operations
#[derive(Error, Debug)]
pub(crate) enum CacheError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Cache error: {0}")]
    GeneralError(String),
}

/// Generic cache entry that stores any result data with a timestamp
#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry<Res> {
    result: Res,
    timestamp: u64,
}

/// Configuration for the generic cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum age of cache entries in seconds. None means cache never expires.
    pub max_age_seconds: Option<u64>,
    /// Whether caching is enabled
    pub enabled: bool,
    /// Whether to skip cache reads and always fetch fresh data (but still write to cache)
    /// Useful for --refresh flags
    pub skip_reads: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl CacheConfig {
    /// Create cache config from environment variables with global prefix
    /// UVM_LIVE_PLATFORM_CACHE_ENABLED and UVM_LIVE_PLATFORM_CACHE_MAX_AGE_SECONDS
    pub fn from_env() -> Self {
        Self::from_env_with_prefix(None)
    }
    
    /// Create cache config from environment variables with API-specific prefix
    /// For example: prefix "FETCH_RELEASE" will look for:
    /// UVM_LIVE_PLATFORM_FETCH_RELEASE_CACHE_ENABLED
    /// UVM_LIVE_PLATFORM_FETCH_RELEASE_CACHE_MAX_AGE_SECONDS
    /// Falls back to global settings if API-specific ones aren't set
    pub fn from_env_with_prefix(api_prefix: Option<&str>) -> Self {
        Self::from_env_with_prefix_and_default(api_prefix, Some(24 * 60 * 60)) // 24 hours default
    }
    
    /// Create cache config with custom default duration
    /// Different APIs can have different default cache durations
    pub fn from_env_with_prefix_and_default(api_prefix: Option<&str>, default_max_age: Option<u64>) -> Self {
        // Always try API-specific first (if provided), then global, then provided default
        let enabled = api_prefix
            .and_then(|prefix| Self::parse_env_bool(&format!("UVM_LIVE_PLATFORM_{}_CACHE_ENABLED", prefix)))
            .or_else(|| Self::parse_env_bool("UVM_LIVE_PLATFORM_CACHE_ENABLED"))
            .unwrap_or(true);
            
        let max_age_seconds = api_prefix
            .and_then(|prefix| Self::parse_env_duration(&format!("UVM_LIVE_PLATFORM_{}_CACHE_MAX_AGE_SECONDS", prefix)))
            .or_else(|| Self::parse_env_duration("UVM_LIVE_PLATFORM_CACHE_MAX_AGE_SECONDS"))
            .unwrap_or(default_max_age);
        
        Self {
            enabled,
            max_age_seconds,
            skip_reads: false,
        }
    }
    
    fn parse_env_bool(key: &str) -> Option<bool> {
        std::env::var(key).ok().and_then(|v| {
            match v.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => None,
            }
        })
    }
    
    fn parse_env_duration(key: &str) -> Option<Option<u64>> {
        std::env::var(key).ok().and_then(|v| {
            let v = v.trim();
            
            // Handle special "never" case
            if v.to_lowercase() == "never" || v.to_lowercase() == "none" {
                return Some(None); // Cache never expires
            }
            
            // Try parsing as human-readable duration first (e.g., "2h", "30m", "1d")
            if let Ok(duration) = humantime::parse_duration(v) {
                return Some(Some(duration.as_secs()));
            }
            
            // Fall back to parsing as raw seconds for backwards compatibility
            v.parse::<u64>().ok().map(Some)
        })
    }
}

/// Generic cache that handles storing and retrieving any data type
#[derive(Debug, Clone)]
pub struct Cache<Opts, Res> {
    config: CacheConfig,
    _phantom: std::marker::PhantomData<(Opts, Res)>,
}

impl<Opts, Res> Cache<Opts, Res> 
where
    Opts: Hash + Serialize + for<'de> Deserialize<'de>,
    Res: Clone + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new Cache with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        Self { 
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new Cache with default configuration
    pub fn default() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Create a Cache with caching disabled
    pub fn disabled() -> Self {
        Self::new(CacheConfig {
            enabled: false,
            max_age_seconds: None,
            skip_reads: false,
        })
    }
    
    /// Create a refresh mode config - always fetch fresh data but still cache it
    pub fn refresh_mode() -> Self {
        Self::new(CacheConfig {
            enabled: true,
            max_age_seconds: Some(24 * 60 * 60), // Default duration when refreshing
            skip_reads: true,
        })
    }

    /// Generate a cache key from any hashable options
    fn generate_cache_key(options: &Opts) -> String {
        let mut hasher = DefaultHasher::new();
        options.hash(&mut hasher);
        format!("cache_{:x}", hasher.finish())
    }

    /// Get the cache directory path
    fn cache_dir() -> Result<PathBuf, std::io::Error> {
        dirs_2::cache_dir()
            .map(|path| path.join("com.github.larusso.unity-version-manager").join("cache"))
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Unable to determine cache directory")
            })
    }

    /// Get the cache file path for given options
    fn cache_file_path(options: &Opts) -> Result<PathBuf, CacheError> {
        let cache_key = Self::generate_cache_key(options);
        let cache_dir = Self::cache_dir().map_err(|e| CacheError::GeneralError(format!("Unable to determine cache directory: {}", e)))?;
        Ok(cache_dir.join(format!("{}.json", cache_key)))
    }

    /// Get current timestamp in seconds since UNIX epoch
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Check if a cache entry is still valid based on timestamp and max_age
    fn is_cache_valid(&self, entry: &CacheEntry<Res>) -> bool {
        if let Some(max_age) = self.config.max_age_seconds {
            let current_time = Self::current_timestamp();
            current_time.saturating_sub(entry.timestamp) <= max_age
        } else {
            true
        }
    }

    /// Retrieve a cached result if it exists and is valid
    fn get(&self, options: &Opts) -> Result<Option<Res>, CacheError> {
        if !self.config.enabled || self.config.skip_reads {
            return Ok(None);
        }

        let cache_file = Self::cache_file_path(options)?;
        
        if !cache_file.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&cache_file)?;
        
        let entry: CacheEntry<Res> = serde_json::from_str(&contents)?;

        if self.is_cache_valid(&entry) {
            Ok(Some(entry.result))
        } else {
            // Cache is expired, remove the file
            let _ = fs::remove_file(&cache_file);
            Ok(None)
        }
    }

    /// Store a result in the cache
    fn put(&self, options: &Opts, result: Res) -> Result<(), CacheError> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_file = Self::cache_file_path(options)?;
        
        // Ensure cache directory exists
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let entry = CacheEntry {
            result,
            timestamp: Self::current_timestamp(),
        };

        let serialized = serde_json::to_string_pretty(&entry)?;

        fs::write(&cache_file, serialized)?;

        Ok(())
    }

    /// Clear all cached entries
    fn clear(&self) -> Result<(), CacheError> {
        let cache_dir = Self::cache_dir().map_err(|e| CacheError::GeneralError(format!("Unable to determine cache directory: {}", e)))?;
        
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
        }
        
        Ok(())
    }

    /// Get cache statistics (number of cached entries)
    fn stats(&self) -> Result<CacheStats, CacheError> {
        let cache_dir = Self::cache_dir().map_err(|e| CacheError::GeneralError(format!("Unable to determine cache directory: {}", e)))?;
        
        if !cache_dir.exists() {
            return Ok(CacheStats { total_entries: 0, valid_entries: 0 });
        }

        let entries = fs::read_dir(&cache_dir)?;

        let mut total_entries = 0;
        let mut valid_entries = 0;

        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "json" {
                        total_entries += 1;
                        
                        // Check if this entry is still valid
                        if let Ok(contents) = fs::read_to_string(entry.path()) {
                            if let Ok(cache_entry) = serde_json::from_str::<CacheEntry<Res>>(&contents) {
                                if self.is_cache_valid(&cache_entry) {
                                    valid_entries += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(CacheStats { total_entries, valid_entries })
    }
}

/// Statistics about the cache
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
}

/// Generic cache middleware that works with any types
#[derive(Debug, Clone)]
pub struct CacheMiddleware<Opts, Res> {
    cache: Cache<Opts, Res>,
}

impl<Opts, Res> CacheMiddleware<Opts, Res>
where
    Opts: Hash + Serialize + for<'de> Deserialize<'de>,
    Res: Clone + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new cache middleware with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Cache::new(config),
        }
    }
    
    /// Create a cache middleware in refresh mode - always fetch fresh data but still cache it
    pub fn refresh_mode() -> Self {
        Self {
            cache: Cache::refresh_mode(),
        }
    }

    /// Create a new cache middleware with default configuration
    pub fn default() -> Self {
        Self {
            cache: Cache::default(),
        }
    }

    /// Create a cache middleware with caching disabled (pass-through)
    pub fn disabled() -> Self {
        Self {
            cache: Cache::disabled(),
        }
    }

    /// Get access to the underlying cache for statistics or management
    pub fn cache(&self) -> &Cache<Opts, Res> {
        &self.cache
    }
}

impl<Opts, Res, Err> Middleware<Opts, Res, Err> for CacheMiddleware<Opts, Res>
where
    Opts: Hash + Serialize + for<'de> Deserialize<'de>,
    Res: Clone + Serialize + for<'de> Deserialize<'de>,
    Err: From<CacheError>,
{
    fn process(
        &self,
        options: &Opts,
        next: &dyn Fn(&Opts) -> Result<Res, Err>,
    ) -> Result<Res, Err> {
        // Try to get from cache first
        if let Ok(Some(cached_result)) = self.cache.get(options) {
            return Ok(cached_result);
        }

        // Not in cache or cache disabled, call the next middleware/handler
        let result = next(options)?;

        // Store the result in cache for future use (ignore cache errors)
        let _ = self.cache.put(options, result.clone());

        Ok(result)
    }
}

