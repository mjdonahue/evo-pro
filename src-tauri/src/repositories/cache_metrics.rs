//! Cache monitoring and metrics
//!
//! This module provides monitoring and metrics collection for cache operations,
//! including hit rate, miss rate, eviction rate, and other performance metrics.

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use crate::{
    error::Result,
    repositories::cache::{CacheKey, CacheStrategy},
};

/// Cache operation type for metrics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheOperation {
    /// Get operation
    Get,
    /// Put operation
    Put,
    /// Remove operation
    Remove,
    /// Clear operation
    Clear,
    /// Eviction (automatic removal due to TTL or capacity)
    Eviction,
}

/// Cache operation result for metrics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheResult {
    /// Hit (found in cache)
    Hit,
    /// Miss (not found in cache)
    Miss,
    /// Success (operation completed successfully)
    Success,
    /// Error (operation failed)
    Error,
}

/// Cache metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Cache name or identifier
    pub name: String,
    /// Cache type (memory, disk, hybrid)
    pub cache_type: String,
    /// Total number of get operations
    pub get_count: u64,
    /// Number of cache hits
    pub hit_count: u64,
    /// Number of cache misses
    pub miss_count: u64,
    /// Total number of put operations
    pub put_count: u64,
    /// Total number of remove operations
    pub remove_count: u64,
    /// Total number of clear operations
    pub clear_count: u64,
    /// Number of evictions
    pub eviction_count: u64,
    /// Number of errors
    pub error_count: u64,
    /// Current number of items in the cache
    pub item_count: usize,
    /// Maximum capacity of the cache
    pub capacity: Option<usize>,
    /// Average response time for get operations (in microseconds)
    pub avg_get_time_us: u64,
    /// Average response time for put operations (in microseconds)
    pub avg_put_time_us: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Memory usage estimate (in bytes)
    pub memory_usage_bytes: Option<u64>,
    /// Disk usage estimate (in bytes)
    pub disk_usage_bytes: Option<u64>,
    /// Time when metrics collection started
    pub start_time: DateTime<Utc>,
    /// Time of the last update
    pub last_updated: DateTime<Utc>,
    /// Time-to-live configuration
    pub ttl: Option<Duration>,
    /// Detailed metrics by time period
    pub time_series: Option<HashMap<String, TimeSeriesMetrics>>,
}

impl CacheMetrics {
    /// Create new cache metrics
    pub fn new(name: impl Into<String>, cache_type: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            cache_type: cache_type.into(),
            get_count: 0,
            hit_count: 0,
            miss_count: 0,
            put_count: 0,
            remove_count: 0,
            clear_count: 0,
            eviction_count: 0,
            error_count: 0,
            item_count: 0,
            capacity: None,
            avg_get_time_us: 0,
            avg_put_time_us: 0,
            hit_rate: 0.0,
            memory_usage_bytes: None,
            disk_usage_bytes: None,
            start_time: now,
            last_updated: now,
            ttl: None,
            time_series: None,
        }
    }

    /// Calculate derived metrics
    pub fn calculate_derived_metrics(&mut self) {
        // Calculate hit rate
        if self.get_count > 0 {
            self.hit_rate = self.hit_count as f64 / self.get_count as f64;
        }

        // Update last updated time
        self.last_updated = Utc::now();
    }

    /// Record an operation
    pub fn record_operation(&mut self, operation: CacheOperation, result: CacheResult, duration_us: u64) {
        match operation {
            CacheOperation::Get => {
                self.get_count += 1;
                match result {
                    CacheResult::Hit => self.hit_count += 1,
                    CacheResult::Miss => self.miss_count += 1,
                    CacheResult::Error => self.error_count += 1,
                    _ => {}
                }

                // Update average get time
                if self.avg_get_time_us == 0 {
                    self.avg_get_time_us = duration_us;
                } else {
                    self.avg_get_time_us = (self.avg_get_time_us + duration_us) / 2;
                }
            }
            CacheOperation::Put => {
                self.put_count += 1;
                if result == CacheResult::Error {
                    self.error_count += 1;
                }

                // Update average put time
                if self.avg_put_time_us == 0 {
                    self.avg_put_time_us = duration_us;
                } else {
                    self.avg_put_time_us = (self.avg_put_time_us + duration_us) / 2;
                }
            }
            CacheOperation::Remove => {
                self.remove_count += 1;
                if result == CacheResult::Error {
                    self.error_count += 1;
                }
            }
            CacheOperation::Clear => {
                self.clear_count += 1;
                if result == CacheResult::Error {
                    self.error_count += 1;
                }
            }
            CacheOperation::Eviction => {
                self.eviction_count += 1;
            }
        }

        // Recalculate derived metrics
        self.calculate_derived_metrics();
    }

    /// Update item count
    pub fn update_item_count(&mut self, count: usize) {
        self.item_count = count;
        self.last_updated = Utc::now();
    }

    /// Set cache capacity
    pub fn set_capacity(&mut self, capacity: Option<usize>) {
        self.capacity = capacity;
    }

    /// Set TTL
    pub fn set_ttl(&mut self, ttl: Option<Duration>) {
        self.ttl = ttl;
    }

    /// Set memory usage
    pub fn set_memory_usage(&mut self, bytes: Option<u64>) {
        self.memory_usage_bytes = bytes;
    }

    /// Set disk usage
    pub fn set_disk_usage(&mut self, bytes: Option<u64>) {
        self.disk_usage_bytes = bytes;
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        let name = self.name.clone();
        let cache_type = self.cache_type.clone();
        let capacity = self.capacity;
        let ttl = self.ttl;
        
        *self = Self::new(name, cache_type);
        self.capacity = capacity;
        self.ttl = ttl;
    }

    /// Get a summary of the metrics
    pub fn summary(&self) -> String {
        format!(
            "Cache '{}' ({}) - Hit Rate: {:.2}%, Items: {}/{}, Ops: {} gets, {} puts, {} removes, {} evictions, {} errors",
            self.name,
            self.cache_type,
            self.hit_rate * 100.0,
            self.item_count,
            self.capacity.map_or_else(|| "∞".to_string(), |c| c.to_string()),
            self.get_count,
            self.put_count,
            self.remove_count,
            self.eviction_count,
            self.error_count
        )
    }
}

/// Time series metrics for tracking cache performance over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesMetrics {
    /// Time period (e.g., "1m", "5m", "1h", "1d")
    pub period: String,
    /// Data points (timestamp -> metrics)
    pub data_points: Vec<TimeSeriesDataPoint>,
    /// Maximum number of data points to keep
    pub max_points: usize,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Hit rate
    pub hit_rate: f64,
    /// Item count
    pub item_count: usize,
    /// Get operations count
    pub get_count: u64,
    /// Put operations count
    pub put_count: u64,
    /// Average get time (microseconds)
    pub avg_get_time_us: u64,
}

impl TimeSeriesMetrics {
    /// Create new time series metrics
    pub fn new(period: impl Into<String>, max_points: usize) -> Self {
        Self {
            period: period.into(),
            data_points: Vec::with_capacity(max_points),
            max_points,
        }
    }

    /// Add a data point
    pub fn add_data_point(&mut self, metrics: &CacheMetrics) {
        let data_point = TimeSeriesDataPoint {
            timestamp: Utc::now(),
            hit_rate: metrics.hit_rate,
            item_count: metrics.item_count,
            get_count: metrics.get_count,
            put_count: metrics.put_count,
            avg_get_time_us: metrics.avg_get_time_us,
        };

        self.data_points.push(data_point);

        // Trim if we have too many points
        if self.data_points.len() > self.max_points {
            self.data_points.remove(0);
        }
    }
}

/// Cache monitor trait for monitoring cache operations
#[async_trait]
pub trait CacheMonitor: Send + Sync + 'static {
    /// Get the metrics for the cache
    async fn get_metrics(&self) -> CacheMetrics;

    /// Reset the metrics
    async fn reset_metrics(&self) -> Result<()>;

    /// Enable time series metrics collection
    async fn enable_time_series(&self, periods: Vec<(String, usize)>) -> Result<()>;

    /// Disable time series metrics collection
    async fn disable_time_series(&self) -> Result<()>;

    /// Export metrics to JSON
    async fn export_metrics_json(&self) -> Result<String>;

    /// Get a metrics summary
    async fn get_metrics_summary(&self) -> String;
}

/// Monitored cache that wraps a cache strategy with metrics collection
pub struct MonitoredCache<K, V, C>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    C: CacheStrategy<K, V>,
{
    /// The underlying cache
    cache: C,
    /// Cache metrics
    metrics: Arc<RwLock<CacheMetrics>>,
    /// Whether to collect time series metrics
    collect_time_series: Arc<RwLock<bool>>,
    /// Time series collection interval
    time_series_interval: Duration,
    /// Time series metrics
    time_series: Arc<RwLock<Option<HashMap<String, TimeSeriesMetrics>>>>,
    /// Phantom data for K and V
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V, C> MonitoredCache<K, V, C>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    C: CacheStrategy<K, V>,
{
    /// Create a new monitored cache
    pub fn new(cache: C, name: impl Into<String>, cache_type: impl Into<String>) -> Self {
        let metrics = Arc::new(RwLock::new(CacheMetrics::new(name, cache_type)));
        
        Self {
            cache,
            metrics,
            collect_time_series: Arc::new(RwLock::new(false)),
            time_series_interval: Duration::from_secs(60), // Default to 1 minute
            time_series: Arc::new(RwLock::new(None)),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Start time series collection
    pub async fn start_time_series_collection(&self) -> Result<()> {
        let metrics_clone = self.metrics.clone();
        let time_series_clone = self.time_series.clone();
        let collect_flag = self.collect_time_series.clone();
        let interval = self.time_series_interval;

        // Set the collection flag
        let mut collect = collect_flag.write().await;
        *collect = true;
        drop(collect);

        // Start the collection task
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Check if we should still collect
                let collect = *collect_flag.read().await;
                if !collect {
                    break;
                }

                // Get current metrics
                let metrics = metrics_clone.read().await.clone();
                
                // Update time series
                let mut time_series = time_series_clone.write().await;
                if let Some(series) = time_series.as_mut() {
                    for (_, period_metrics) in series.iter_mut() {
                        period_metrics.add_data_point(&metrics);
                    }
                }
            }
        });

        Ok(())
    }

    /// Update metrics with the current cache state
    async fn update_metrics(&self) -> Result<()> {
        let item_count = self.cache.len().await;
        
        let mut metrics = self.metrics.write().await;
        metrics.update_item_count(item_count);
        
        Ok(())
    }
}

#[async_trait]
impl<K, V, C> CacheStrategy<K, V> for MonitoredCache<K, V, C>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    C: CacheStrategy<K, V>,
{
    #[instrument(skip(self))]
    async fn get(&self, key: &K) -> Option<V> {
        let start = Instant::now();
        let result = self.cache.get(key).await;
        let duration = start.elapsed();
        
        // Record the operation
        let op_result = if result.is_some() {
            CacheResult::Hit
        } else {
            CacheResult::Miss
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.record_operation(
            CacheOperation::Get,
            op_result,
            duration.as_micros() as u64,
        );
        
        result
    }
    
    #[instrument(skip(self, value))]
    async fn put(&self, key: K, value: V) -> Result<()> {
        let start = Instant::now();
        let result = self.cache.put(key, value).await;
        let duration = start.elapsed();
        
        // Record the operation
        let op_result = if result.is_ok() {
            CacheResult::Success
        } else {
            CacheResult::Error
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.record_operation(
            CacheOperation::Put,
            op_result,
            duration.as_micros() as u64,
        );
        
        // Update metrics with current cache state
        drop(metrics);
        self.update_metrics().await?;
        
        result
    }
    
    #[instrument(skip(self))]
    async fn remove(&self, key: &K) -> Result<()> {
        let start = Instant::now();
        let result = self.cache.remove(key).await;
        let duration = start.elapsed();
        
        // Record the operation
        let op_result = if result.is_ok() {
            CacheResult::Success
        } else {
            CacheResult::Error
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.record_operation(
            CacheOperation::Remove,
            op_result,
            duration.as_micros() as u64,
        );
        
        // Update metrics with current cache state
        drop(metrics);
        self.update_metrics().await?;
        
        result
    }
    
    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        let start = Instant::now();
        let result = self.cache.clear().await;
        let duration = start.elapsed();
        
        // Record the operation
        let op_result = if result.is_ok() {
            CacheResult::Success
        } else {
            CacheResult::Error
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.record_operation(
            CacheOperation::Clear,
            op_result,
            duration.as_micros() as u64,
        );
        
        // Update metrics with current cache state
        drop(metrics);
        self.update_metrics().await?;
        
        result
    }
    
    async fn len(&self) -> usize {
        self.cache.len().await
    }
}

#[async_trait]
impl<K, V, C> CacheMonitor for MonitoredCache<K, V, C>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    C: CacheStrategy<K, V>,
{
    async fn get_metrics(&self) -> CacheMetrics {
        // Update metrics with current cache state
        let _ = self.update_metrics().await;
        
        // Return a clone of the metrics
        self.metrics.read().await.clone()
    }
    
    async fn reset_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.reset();
        
        // Update with current cache state
        let item_count = self.cache.len().await;
        metrics.update_item_count(item_count);
        
        Ok(())
    }
    
    async fn enable_time_series(&self, periods: Vec<(String, usize)>) -> Result<()> {
        // Initialize time series collection
        let mut time_series = self.time_series.write().await;
        let mut series_map = HashMap::new();
        
        for (period, max_points) in periods {
            series_map.insert(period.clone(), TimeSeriesMetrics::new(period, max_points));
        }
        
        *time_series = Some(series_map);
        drop(time_series);
        
        // Start collection
        self.start_time_series_collection().await?;
        
        Ok(())
    }
    
    async fn disable_time_series(&self) -> Result<()> {
        let mut collect = self.collect_time_series.write().await;
        *collect = false;
        Ok(())
    }
    
    async fn export_metrics_json(&self) -> Result<String> {
        let metrics = self.get_metrics().await;
        Ok(serde_json::to_string_pretty(&metrics)?)
    }
    
    async fn get_metrics_summary(&self) -> String {
        let metrics = self.metrics.read().await;
        metrics.summary()
    }
}

/// Metrics collector for aggregating metrics from multiple caches
pub struct MetricsCollector {
    /// Monitored caches
    caches: Arc<RwLock<HashMap<String, Arc<dyn CacheMonitor>>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a cache for monitoring
    pub async fn register_cache(&self, name: impl Into<String>, cache: Arc<dyn CacheMonitor>) {
        let mut caches = self.caches.write().await;
        caches.insert(name.into(), cache);
    }
    
    /// Unregister a cache
    pub async fn unregister_cache(&self, name: &str) {
        let mut caches = self.caches.write().await;
        caches.remove(name);
    }
    
    /// Get metrics for all caches
    pub async fn get_all_metrics(&self) -> HashMap<String, CacheMetrics> {
        let caches = self.caches.read().await;
        let mut metrics = HashMap::new();
        
        for (name, cache) in caches.iter() {
            metrics.insert(name.clone(), cache.get_metrics().await);
        }
        
        metrics
    }
    
    /// Get metrics for a specific cache
    pub async fn get_cache_metrics(&self, name: &str) -> Option<CacheMetrics> {
        let caches = self.caches.read().await;
        caches.get(name).map(|cache| async move { cache.get_metrics().await }).await
    }
    
    /// Reset metrics for all caches
    pub async fn reset_all_metrics(&self) -> Result<()> {
        let caches = self.caches.read().await;
        
        for cache in caches.values() {
            cache.reset_metrics().await?;
        }
        
        Ok(())
    }
    
    /// Reset metrics for a specific cache
    pub async fn reset_cache_metrics(&self, name: &str) -> Result<()> {
        let caches = self.caches.read().await;
        
        if let Some(cache) = caches.get(name) {
            cache.reset_metrics().await?;
            Ok(())
        } else {
            Err(crate::error::AppError::not_found("Cache", name))
        }
    }
    
    /// Export metrics for all caches as JSON
    pub async fn export_all_metrics_json(&self) -> Result<String> {
        let metrics = self.get_all_metrics().await;
        Ok(serde_json::to_string_pretty(&metrics)?)
    }
    
    /// Get a summary of all cache metrics
    pub async fn get_summary(&self) -> String {
        let caches = self.caches.read().await;
        let mut summary = String::new();
        
        summary.push_str(&format!("Cache Metrics Summary ({} caches):\n", caches.len()));
        
        for (name, cache) in caches.iter() {
            let cache_summary = cache.get_metrics_summary().await;
            summary.push_str(&format!("- {}: {}\n", name, cache_summary));
        }
        
        summary
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating monitored caches
pub struct MonitoredCacheFactory;

impl MonitoredCacheFactory {
    /// Create a monitored memory cache
    pub fn memory_cache<K, V>(
        name: impl Into<String>,
        max_entries: usize,
        ttl: Option<Duration>,
    ) -> impl CacheStrategy<K, V> + CacheMonitor
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
    {
        let cache = crate::repositories::cache::CacheFactory::memory_cache(max_entries, ttl);
        let monitored = MonitoredCache::new(cache, name, "memory");
        
        // Set capacity and TTL in metrics
        tokio::spawn(async move {
            let mut metrics = monitored.metrics.write().await;
            metrics.set_capacity(Some(max_entries));
            metrics.set_ttl(ttl);
        });
        
        monitored
    }
    
    /// Create a monitored disk cache
    pub async fn disk_cache<K, V>(
        name: impl Into<String>,
        cache_dir: std::path::PathBuf,
        ttl: Option<Duration>,
    ) -> Result<impl CacheStrategy<K, V> + CacheMonitor>
    where
        K: CacheKey + Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    {
        let cache = crate::repositories::cache::CacheFactory::disk_cache(cache_dir.clone(), ttl).await?;
        let monitored = MonitoredCache::new(cache, name, "disk");
        
        // Set TTL in metrics
        let metrics_clone = monitored.metrics.clone();
        tokio::spawn(async move {
            let mut metrics = metrics_clone.write().await;
            metrics.set_ttl(ttl);
            
            // Estimate disk usage
            if let Ok(mut size) = tokio::fs::metadata(&cache_dir).await {
                if size.is_dir() {
                    // Sum up the size of all files in the directory
                    let mut total_size = 0;
                    let mut entries = tokio::fs::read_dir(&cache_dir).await.unwrap();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        if let Ok(metadata) = entry.metadata().await {
                            total_size += metadata.len();
                        }
                    }
                    metrics.set_disk_usage(Some(total_size));
                }
            }
        });
        
        Ok(monitored)
    }
    
    /// Create a monitored hybrid cache
    pub async fn hybrid_cache<K, V>(
        name: impl Into<String>,
        max_memory_entries: usize,
        memory_ttl: Option<Duration>,
        cache_dir: std::path::PathBuf,
        disk_ttl: Option<Duration>,
    ) -> Result<impl CacheStrategy<K, V> + CacheMonitor>
    where
        K: CacheKey + Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    {
        let cache = crate::repositories::cache::CacheFactory::hybrid_cache(
            max_memory_entries,
            memory_ttl,
            cache_dir.clone(),
            disk_ttl,
        ).await?;
        
        let monitored = MonitoredCache::new(cache, name, "hybrid");
        
        // Set capacity and TTL in metrics
        let metrics_clone = monitored.metrics.clone();
        tokio::spawn(async move {
            let mut metrics = metrics_clone.write().await;
            metrics.set_capacity(Some(max_memory_entries));
            metrics.set_ttl(memory_ttl.or(disk_ttl));
            
            // Estimate memory usage (rough approximation)
            let memory_estimate = max_memory_entries as u64 * 1024; // Assume 1KB per entry
            metrics.set_memory_usage(Some(memory_estimate));
            
            // Estimate disk usage
            if let Ok(metadata) = tokio::fs::metadata(&cache_dir).await {
                if metadata.is_dir() {
                    // Sum up the size of all files in the directory
                    let mut total_size = 0;
                    let mut entries = tokio::fs::read_dir(&cache_dir).await.unwrap();
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        if let Ok(metadata) = entry.metadata().await {
                            total_size += metadata.len();
                        }
                    }
                    metrics.set_disk_usage(Some(total_size));
                }
            }
        });
        
        Ok(monitored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_monitored_memory_cache() -> Result<()> {
        // Create a monitored memory cache
        let cache = MonitoredCacheFactory::memory_cache::<String, String>(
            "test_memory_cache",
            100,
            Some(Duration::from_secs(60)),
        );
        
        // Put some values
        cache.put("key1".to_string(), "value1".to_string()).await?;
        cache.put("key2".to_string(), "value2".to_string()).await?;
        
        // Get values (one hit, one miss)
        let _ = cache.get(&"key1".to_string()).await;
        let _ = cache.get(&"key3".to_string()).await;
        
        // Get metrics
        let metrics = cache.get_metrics().await;
        
        // Verify metrics
        assert_eq!(metrics.name, "test_memory_cache");
        assert_eq!(metrics.cache_type, "memory");
        assert_eq!(metrics.get_count, 2);
        assert_eq!(metrics.hit_count, 1);
        assert_eq!(metrics.miss_count, 1);
        assert_eq!(metrics.put_count, 2);
        assert_eq!(metrics.item_count, 2);
        assert_eq!(metrics.hit_rate, 0.5);
        
        // Test metrics summary
        let summary = cache.get_metrics_summary().await;
        assert!(summary.contains("test_memory_cache"));
        assert!(summary.contains("50.00%"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_monitored_disk_cache() -> Result<()> {
        // Create a temporary directory for the cache
        let temp_dir = tempdir()?;
        let cache_dir = temp_dir.path().to_path_buf();
        
        // Create a monitored disk cache
        let cache = MonitoredCacheFactory::disk_cache::<String, String>(
            "test_disk_cache",
            cache_dir,
            Some(Duration::from_secs(60)),
        ).await?;
        
        // Put some values
        cache.put("key1".to_string(), "value1".to_string()).await?;
        cache.put("key2".to_string(), "value2".to_string()).await?;
        
        // Get values (one hit, one miss)
        let _ = cache.get(&"key1".to_string()).await;
        let _ = cache.get(&"key3".to_string()).await;
        
        // Get metrics
        let metrics = cache.get_metrics().await;
        
        // Verify metrics
        assert_eq!(metrics.name, "test_disk_cache");
        assert_eq!(metrics.cache_type, "disk");
        assert_eq!(metrics.get_count, 2);
        assert_eq!(metrics.hit_count, 1);
        assert_eq!(metrics.miss_count, 1);
        assert_eq!(metrics.put_count, 2);
        assert_eq!(metrics.item_count, 2);
        assert_eq!(metrics.hit_rate, 0.5);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_metrics_collector() -> Result<()> {
        // Create a metrics collector
        let collector = MetricsCollector::new();
        
        // Create two monitored caches
        let cache1 = MonitoredCacheFactory::memory_cache::<String, String>(
            "cache1",
            100,
            Some(Duration::from_secs(60)),
        );
        
        let cache2 = MonitoredCacheFactory::memory_cache::<String, String>(
            "cache2",
            200,
            Some(Duration::from_secs(120)),
        );
        
        // Register the caches
        collector.register_cache("cache1", Arc::new(cache1.clone())).await;
        collector.register_cache("cache2", Arc::new(cache2.clone())).await;
        
        // Put some values in both caches
        cache1.put("key1".to_string(), "value1".to_string()).await?;
        cache2.put("key2".to_string(), "value2".to_string()).await?;
        
        // Get values from both caches
        let _ = cache1.get(&"key1".to_string()).await;
        let _ = cache2.get(&"key2".to_string()).await;
        
        // Get all metrics
        let all_metrics = collector.get_all_metrics().await;
        
        // Verify metrics
        assert_eq!(all_metrics.len(), 2);
        assert_eq!(all_metrics["cache1"].name, "cache1");
        assert_eq!(all_metrics["cache2"].name, "cache2");
        assert_eq!(all_metrics["cache1"].hit_count, 1);
        assert_eq!(all_metrics["cache2"].hit_count, 1);
        
        // Get summary
        let summary = collector.get_summary().await;
        assert!(summary.contains("cache1"));
        assert!(summary.contains("cache2"));
        
        // Reset metrics for one cache
        collector.reset_cache_metrics("cache1").await?;
        
        // Verify reset
        let cache1_metrics = collector.get_cache_metrics("cache1").await.unwrap();
        assert_eq!(cache1_metrics.hit_count, 0);
        
        // Cache2 should still have its metrics
        let cache2_metrics = collector.get_cache_metrics("cache2").await.unwrap();
        assert_eq!(cache2_metrics.hit_count, 1);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_time_series_metrics() -> Result<()> {
        // Create a monitored memory cache
        let cache = MonitoredCacheFactory::memory_cache::<String, String>(
            "time_series_test",
            100,
            Some(Duration::from_secs(60)),
        );
        
        // Enable time series collection
        cache.enable_time_series(vec![
            ("1m".to_string(), 60),  // 1 minute with 60 data points (1 per second)
            ("5m".to_string(), 30),  // 5 minutes with 30 data points (1 per 10 seconds)
        ]).await?;
        
        // Put and get some values
        for i in 0..5 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cache.put(key.clone(), value).await?;
            let _ = cache.get(&key).await;
        }
        
        // Wait a bit for time series collection
        sleep(Duration::from_millis(100)).await;
        
        // Get metrics
        let metrics = cache.get_metrics().await;
        
        // Verify time series exists
        assert!(metrics.time_series.is_some());
        let time_series = metrics.time_series.unwrap();
        assert!(time_series.contains_key("1m"));
        assert!(time_series.contains_key("5m"));
        
        // Disable time series collection
        cache.disable_time_series().await?;
        
        Ok(())
    }
}