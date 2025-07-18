//! Cache warming strategies for repositories
//!
//! This module provides cache warming strategies for repositories, including
//! eager loading, predictive loading, and scheduled warming.

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{sync::RwLock, task::JoinHandle, time};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    error::Result,
    repositories::cache::{CacheKey, CacheStrategy},
};

/// Cache warming strategy trait
#[async_trait]
pub trait CacheWarmer<K, V>: Send + Sync + 'static
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Warm the cache with the given data loader
    async fn warm(&self, cache: &impl CacheStrategy<K, V>, loader: &impl DataLoader<K, V>) -> Result<usize>;
    
    /// Get the name of the warming strategy
    fn name(&self) -> &str;
    
    /// Get the configuration of the warming strategy
    fn config(&self) -> WarmingConfig;
}

/// Data loader trait for loading data into the cache
#[async_trait]
pub trait DataLoader<K, V>: Send + Sync + 'static
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Load a single item by key
    async fn load_one(&self, key: &K) -> Result<Option<V>>;
    
    /// Load multiple items by keys
    async fn load_many(&self, keys: &[K]) -> Result<HashMap<K, V>>;
    
    /// Load all items
    async fn load_all(&self) -> Result<HashMap<K, V>>;
    
    /// Get keys for frequently accessed items
    async fn get_frequent_keys(&self, limit: usize) -> Result<Vec<K>>;
    
    /// Get keys for recently accessed items
    async fn get_recent_keys(&self, limit: usize) -> Result<Vec<K>>;
    
    /// Get keys for items that match the given pattern
    async fn get_pattern_keys(&self, pattern: &str) -> Result<Vec<K>>;
}

/// Cache warming configuration
#[derive(Debug, Clone)]
pub struct WarmingConfig {
    /// Maximum number of items to warm
    pub max_items: usize,
    /// Batch size for loading items
    pub batch_size: usize,
    /// Warming interval for scheduled warming
    pub interval: Option<Duration>,
    /// Whether to warm the cache on startup
    pub warm_on_startup: bool,
    /// Whether to warm the cache in the background
    pub background: bool,
    /// Patterns to include in warming
    pub include_patterns: Vec<String>,
    /// Patterns to exclude from warming
    pub exclude_patterns: Vec<String>,
}

impl Default for WarmingConfig {
    fn default() -> Self {
        Self {
            max_items: 1000,
            batch_size: 100,
            interval: None,
            warm_on_startup: true,
            background: true,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }
}

/// Eager loading cache warming strategy
pub struct EagerWarmer {
    /// Warming configuration
    config: WarmingConfig,
}

impl EagerWarmer {
    /// Create a new eager warming strategy
    pub fn new(config: WarmingConfig) -> Self {
        Self { config }
    }
    
    /// Create a new eager warming strategy with default configuration
    pub fn default() -> Self {
        Self::new(WarmingConfig::default())
    }
}

#[async_trait]
impl<K, V> CacheWarmer<K, V> for EagerWarmer
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, cache, loader))]
    async fn warm(&self, cache: &impl CacheStrategy<K, V>, loader: &impl DataLoader<K, V>) -> Result<usize> {
        info!("Starting eager cache warming");
        
        // Load all items
        let items = loader.load_all().await?;
        let total_items = items.len();
        
        // Limit the number of items if needed
        let items_to_warm = if total_items > self.config.max_items {
            info!("Limiting cache warming to {} items (out of {})", self.config.max_items, total_items);
            items.into_iter().take(self.config.max_items).collect::<HashMap<_, _>>()
        } else {
            items
        };
        
        // Warm the cache in batches
        let mut warmed_count = 0;
        let mut batch = Vec::with_capacity(self.config.batch_size);
        
        for (key, value) in items_to_warm {
            batch.push((key, value));
            
            if batch.len() >= self.config.batch_size {
                for (k, v) in batch.drain(..) {
                    cache.put(k, v).await?;
                    warmed_count += 1;
                }
            }
        }
        
        // Warm any remaining items
        for (k, v) in batch {
            cache.put(k, v).await?;
            warmed_count += 1;
        }
        
        info!("Completed eager cache warming: {} items warmed", warmed_count);
        Ok(warmed_count)
    }
    
    fn name(&self) -> &str {
        "eager"
    }
    
    fn config(&self) -> WarmingConfig {
        self.config.clone()
    }
}

/// Predictive loading cache warming strategy
pub struct PredictiveWarmer {
    /// Warming configuration
    config: WarmingConfig,
    /// Access frequency threshold
    frequency_threshold: u64,
    /// Recent access threshold
    recency_threshold: DateTime<Utc>,
}

impl PredictiveWarmer {
    /// Create a new predictive warming strategy
    pub fn new(
        config: WarmingConfig,
        frequency_threshold: u64,
        recency_threshold: Duration,
    ) -> Self {
        Self {
            config,
            frequency_threshold,
            recency_threshold: Utc::now() - chrono::Duration::from_std(recency_threshold).unwrap_or_default(),
        }
    }
    
    /// Create a new predictive warming strategy with default configuration
    pub fn default() -> Self {
        Self::new(
            WarmingConfig::default(),
            10, // Access at least 10 times
            Duration::from_secs(60 * 60 * 24), // Accessed within the last 24 hours
        )
    }
}

#[async_trait]
impl<K, V> CacheWarmer<K, V> for PredictiveWarmer
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, cache, loader))]
    async fn warm(&self, cache: &impl CacheStrategy<K, V>, loader: &impl DataLoader<K, V>) -> Result<usize> {
        info!("Starting predictive cache warming");
        
        // Get frequently accessed keys
        let frequent_keys = loader.get_frequent_keys(self.config.max_items).await?;
        
        // Get recently accessed keys
        let recent_keys = loader.get_recent_keys(self.config.max_items).await?;
        
        // Combine and deduplicate keys
        let mut keys_to_warm = HashSet::new();
        for key in frequent_keys {
            keys_to_warm.insert(key);
            if keys_to_warm.len() >= self.config.max_items {
                break;
            }
        }
        
        for key in recent_keys {
            keys_to_warm.insert(key);
            if keys_to_warm.len() >= self.config.max_items {
                break;
            }
        }
        
        let keys_vec: Vec<_> = keys_to_warm.into_iter().collect();
        
        // Warm the cache in batches
        let mut warmed_count = 0;
        let mut current_index = 0;
        
        while current_index < keys_vec.len() {
            let end_index = (current_index + self.config.batch_size).min(keys_vec.len());
            let batch = &keys_vec[current_index..end_index];
            
            let items = loader.load_many(batch).await?;
            
            for (key, value) in items {
                cache.put(key, value).await?;
                warmed_count += 1;
            }
            
            current_index = end_index;
        }
        
        info!("Completed predictive cache warming: {} items warmed", warmed_count);
        Ok(warmed_count)
    }
    
    fn name(&self) -> &str {
        "predictive"
    }
    
    fn config(&self) -> WarmingConfig {
        self.config.clone()
    }
}

/// Scheduled cache warming strategy
pub struct ScheduledWarmer<K, V, W, L>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    W: CacheWarmer<K, V>,
    L: DataLoader<K, V>,
{
    /// Inner warmer
    inner_warmer: W,
    /// Data loader
    loader: Arc<L>,
    /// Cache
    cache: Arc<dyn CacheStrategy<K, V>>,
    /// Task handle for the warming task
    task_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Phantom data for K and V
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, W, L> ScheduledWarmer<K, V, W, L>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    W: CacheWarmer<K, V>,
    L: DataLoader<K, V>,
{
    /// Create a new scheduled warming strategy
    pub fn new(
        inner_warmer: W,
        loader: L,
        cache: impl CacheStrategy<K, V> + 'static,
    ) -> Self {
        Self {
            inner_warmer,
            loader: Arc::new(loader),
            cache: Arc::new(cache),
            task_handle: Arc::new(RwLock::new(None)),
            _phantom: PhantomData,
        }
    }
    
    /// Start the scheduled warming task
    pub async fn start(&self) -> Result<()> {
        let config = self.inner_warmer.config();
        
        if let Some(interval) = config.interval {
            let inner_warmer = self.inner_warmer.clone();
            let loader = self.loader.clone();
            let cache = self.cache.clone();
            
            // Warm on startup if configured
            if config.warm_on_startup {
                info!("Performing initial cache warming");
                if let Err(e) = inner_warmer.warm(cache.as_ref(), loader.as_ref()).await {
                    error!("Initial cache warming failed: {}", e);
                }
            }
            
            // Start the scheduled task
            let task = tokio::spawn(async move {
                let mut interval_timer = time::interval(interval);
                
                loop {
                    interval_timer.tick().await;
                    info!("Starting scheduled cache warming");
                    
                    match inner_warmer.warm(cache.as_ref(), loader.as_ref()).await {
                        Ok(count) => {
                            info!("Scheduled cache warming completed: {} items warmed", count);
                        }
                        Err(e) => {
                            error!("Scheduled cache warming failed: {}", e);
                        }
                    }
                }
            });
            
            let mut handle = self.task_handle.write().await;
            *handle = Some(task);
        }
        
        Ok(())
    }
    
    /// Stop the scheduled warming task
    pub async fn stop(&self) -> Result<()> {
        let mut handle = self.task_handle.write().await;
        
        if let Some(task) = handle.take() {
            task.abort();
            info!("Scheduled cache warming task stopped");
        }
        
        Ok(())
    }
}

impl<K, V, W, L> Clone for ScheduledWarmer<K, V, W, L>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    W: CacheWarmer<K, V> + Clone,
    L: DataLoader<K, V>,
{
    fn clone(&self) -> Self {
        Self {
            inner_warmer: self.inner_warmer.clone(),
            loader: self.loader.clone(),
            cache: self.cache.clone(),
            task_handle: self.task_handle.clone(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<K, V, W, L> CacheWarmer<K, V> for ScheduledWarmer<K, V, W, L>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    W: CacheWarmer<K, V> + Clone,
    L: DataLoader<K, V>,
{
    #[instrument(skip(self, cache, loader))]
    async fn warm(&self, cache: &impl CacheStrategy<K, V>, loader: &impl DataLoader<K, V>) -> Result<usize> {
        // Delegate to the inner warmer
        self.inner_warmer.warm(cache, loader).await
    }
    
    fn name(&self) -> &str {
        "scheduled"
    }
    
    fn config(&self) -> WarmingConfig {
        self.inner_warmer.config()
    }
}

/// Pattern-based cache warming strategy
pub struct PatternWarmer {
    /// Warming configuration
    config: WarmingConfig,
    /// Patterns to include
    include_patterns: Vec<String>,
    /// Patterns to exclude
    exclude_patterns: Vec<String>,
}

impl PatternWarmer {
    /// Create a new pattern-based warming strategy
    pub fn new(
        config: WarmingConfig,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
    ) -> Self {
        Self {
            config,
            include_patterns,
            exclude_patterns,
        }
    }
    
    /// Create a new pattern-based warming strategy with default configuration
    pub fn default() -> Self {
        Self::new(
            WarmingConfig::default(),
            vec!["*".to_string()], // Include everything by default
            Vec::new(),            // Exclude nothing by default
        )
    }
    
    /// Check if a key matches the patterns
    fn matches_patterns<K: Debug>(&self, key: &K) -> bool {
        let key_str = format!("{:?}", key);
        
        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if pattern_match(&key_str, pattern) {
                return false;
            }
        }
        
        // Then check include patterns
        for pattern in &self.include_patterns {
            if pattern_match(&key_str, pattern) {
                return true;
            }
        }
        
        // If no include patterns match, exclude by default
        false
    }
}

#[async_trait]
impl<K, V> CacheWarmer<K, V> for PatternWarmer
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, cache, loader))]
    async fn warm(&self, cache: &impl CacheStrategy<K, V>, loader: &impl DataLoader<K, V>) -> Result<usize> {
        info!("Starting pattern-based cache warming");
        
        let mut warmed_count = 0;
        
        // Process each include pattern
        for pattern in &self.include_patterns {
            let keys = loader.get_pattern_keys(pattern).await?;
            
            // Filter keys based on exclude patterns
            let filtered_keys: Vec<_> = keys
                .into_iter()
                .filter(|key| !self.exclude_patterns.iter().any(|p| pattern_match(&format!("{:?}", key), p)))
                .take(self.config.max_items - warmed_count)
                .collect();
            
            if filtered_keys.is_empty() {
                continue;
            }
            
            // Load and cache the filtered keys in batches
            let mut current_index = 0;
            
            while current_index < filtered_keys.len() {
                let end_index = (current_index + self.config.batch_size).min(filtered_keys.len());
                let batch = &filtered_keys[current_index..end_index];
                
                let items = loader.load_many(batch).await?;
                
                for (key, value) in items {
                    cache.put(key, value).await?;
                    warmed_count += 1;
                    
                    if warmed_count >= self.config.max_items {
                        info!("Reached maximum items limit for cache warming");
                        return Ok(warmed_count);
                    }
                }
                
                current_index = end_index;
            }
        }
        
        info!("Completed pattern-based cache warming: {} items warmed", warmed_count);
        Ok(warmed_count)
    }
    
    fn name(&self) -> &str {
        "pattern"
    }
    
    fn config(&self) -> WarmingConfig {
        self.config.clone()
    }
}

/// Cache warming factory for creating different types of cache warmers
pub struct CacheWarmingFactory;

impl CacheWarmingFactory {
    /// Create an eager warming strategy
    pub fn eager_warmer(config: Option<WarmingConfig>) -> impl CacheWarmer<Uuid, Vec<u8>> {
        EagerWarmer::new(config.unwrap_or_default())
    }
    
    /// Create a predictive warming strategy
    pub fn predictive_warmer(
        config: Option<WarmingConfig>,
        frequency_threshold: Option<u64>,
        recency_threshold: Option<Duration>,
    ) -> impl CacheWarmer<Uuid, Vec<u8>> {
        PredictiveWarmer::new(
            config.unwrap_or_default(),
            frequency_threshold.unwrap_or(10),
            recency_threshold.unwrap_or_else(|| Duration::from_secs(60 * 60 * 24)),
        )
    }
    
    /// Create a pattern-based warming strategy
    pub fn pattern_warmer(
        config: Option<WarmingConfig>,
        include_patterns: Option<Vec<String>>,
        exclude_patterns: Option<Vec<String>>,
    ) -> impl CacheWarmer<Uuid, Vec<u8>> {
        PatternWarmer::new(
            config.unwrap_or_default(),
            include_patterns.unwrap_or_else(|| vec!["*".to_string()]),
            exclude_patterns.unwrap_or_default(),
        )
    }
    
    /// Create a scheduled warming strategy with an inner warmer
    pub async fn scheduled_warmer<K, V, W, L>(
        inner_warmer: W,
        loader: L,
        cache: impl CacheStrategy<K, V> + 'static,
    ) -> Result<ScheduledWarmer<K, V, W, L>>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
        W: CacheWarmer<K, V> + Clone,
        L: DataLoader<K, V>,
    {
        let warmer = ScheduledWarmer::new(inner_warmer, loader, cache);
        warmer.start().await?;
        Ok(warmer)
    }
}

/// Simple pattern matching function (supports * wildcard)
fn pattern_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    
    let pattern_parts: Vec<&str> = pattern.split('*').collect();
    
    if pattern_parts.len() == 1 {
        // No wildcards, exact match
        return text == pattern;
    }
    
    let mut text_pos = 0;
    
    // Check if pattern starts with *
    if !pattern.starts_with('*') && !text.starts_with(pattern_parts[0]) {
        return false;
    }
    
    // Check if pattern ends with *
    if !pattern.ends_with('*') && !text.ends_with(pattern_parts.last().unwrap()) {
        return false;
    }
    
    // Match each part
    for (i, part) in pattern_parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        
        match text[text_pos..].find(part) {
            Some(pos) => {
                text_pos += pos + part.len();
                
                // If this is the last part and it doesn't end with *, ensure we've reached the end
                if i == pattern_parts.len() - 1 && !pattern.ends_with('*') && text_pos != text.len() {
                    return false;
                }
            }
            None => return false,
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    use crate::repositories::cache::MemoryCache;
    
    // Mock data loader for testing
    struct MockDataLoader {
        data: HashMap<Uuid, Vec<u8>>,
        access_counts: HashMap<Uuid, u64>,
        access_times: HashMap<Uuid, DateTime<Utc>>,
    }
    
    impl MockDataLoader {
        fn new() -> Self {
            let mut data = HashMap::new();
            let mut access_counts = HashMap::new();
            let mut access_times = HashMap::new();
            
            // Create some test data
            for i in 0..100 {
                let id = Uuid::new_v4();
                let value = vec![i as u8; 10];
                data.insert(id, value);
                
                // Simulate some access patterns
                let count = if i < 20 { 20 } else { i / 5 };
                access_counts.insert(id, count);
                
                let time = Utc::now() - chrono::Duration::hours(i as i64);
                access_times.insert(id, time);
            }
            
            Self {
                data,
                access_counts,
                access_times,
            }
        }
    }
    
    #[async_trait]
    impl DataLoader<Uuid, Vec<u8>> for MockDataLoader {
        async fn load_one(&self, key: &Uuid) -> Result<Option<Vec<u8>>> {
            Ok(self.data.get(key).cloned())
        }
        
        async fn load_many(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Vec<u8>>> {
            let mut result = HashMap::new();
            for key in keys {
                if let Some(value) = self.data.get(key) {
                    result.insert(*key, value.clone());
                }
            }
            Ok(result)
        }
        
        async fn load_all(&self) -> Result<HashMap<Uuid, Vec<u8>>> {
            Ok(self.data.clone())
        }
        
        async fn get_frequent_keys(&self, limit: usize) -> Result<Vec<Uuid>> {
            let mut keys: Vec<_> = self.access_counts.iter().collect();
            keys.sort_by(|a, b| b.1.cmp(a.1));
            
            Ok(keys.into_iter().take(limit).map(|(k, _)| *k).collect())
        }
        
        async fn get_recent_keys(&self, limit: usize) -> Result<Vec<Uuid>> {
            let mut keys: Vec<_> = self.access_times.iter().collect();
            keys.sort_by(|a, b| b.1.cmp(a.1));
            
            Ok(keys.into_iter().take(limit).map(|(k, _)| *k).collect())
        }
        
        async fn get_pattern_keys(&self, _pattern: &str) -> Result<Vec<Uuid>> {
            // For simplicity, just return all keys
            Ok(self.data.keys().cloned().collect())
        }
    }
    
    #[tokio::test]
    async fn test_eager_warmer() -> Result<()> {
        let loader = MockDataLoader::new();
        let cache = MemoryCache::<Uuid, Vec<u8>>::new(1000, None);
        
        let config = WarmingConfig {
            max_items: 50,
            batch_size: 10,
            ..Default::default()
        };
        
        let warmer = EagerWarmer::new(config);
        let count = warmer.warm(&cache, &loader).await?;
        
        assert_eq!(count, 50);
        assert_eq!(cache.len().await, 50);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_predictive_warmer() -> Result<()> {
        let loader = MockDataLoader::new();
        let cache = MemoryCache::<Uuid, Vec<u8>>::new(1000, None);
        
        let config = WarmingConfig {
            max_items: 30,
            batch_size: 10,
            ..Default::default()
        };
        
        let warmer = PredictiveWarmer::new(
            config,
            5,
            Duration::from_secs(60 * 60 * 48), // 48 hours
        );
        
        let count = warmer.warm(&cache, &loader).await?;
        
        // Should warm up to 30 items (our max_items limit)
        assert!(count <= 30);
        assert_eq!(cache.len().await, count);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_pattern_matching() {
        // Test exact match
        assert!(pattern_match("hello", "hello"));
        assert!(!pattern_match("hello", "world"));
        
        // Test wildcard at beginning
        assert!(pattern_match("hello world", "*world"));
        assert!(!pattern_match("hello world", "*universe"));
        
        // Test wildcard at end
        assert!(pattern_match("hello world", "hello*"));
        assert!(!pattern_match("hello world", "goodbye*"));
        
        // Test wildcard in middle
        assert!(pattern_match("hello beautiful world", "hello*world"));
        assert!(!pattern_match("hello beautiful universe", "hello*world"));
        
        // Test multiple wildcards
        assert!(pattern_match("hello beautiful world", "*beautiful*"));
        assert!(pattern_match("hello beautiful world", "hello*beautiful*world"));
        assert!(!pattern_match("goodbye cruel world", "hello*beautiful*world"));
    }
}