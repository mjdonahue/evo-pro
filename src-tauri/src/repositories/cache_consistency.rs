//! Cache consistency mechanisms for repositories
//!
//! This module provides strategies for maintaining cache consistency,
//! including write-through, write-behind, and invalidation-based approaches.
//! It supports eventual consistency across distributed caches.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    marker::PhantomData,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::{broadcast, mpsc, oneshot, RwLock},
    task::JoinHandle,
    time,
};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    error::Result,
    repositories::cache::{CacheKey, CacheStrategy},
};

/// Cache consistency strategy trait
#[async_trait]
pub trait CacheConsistencyStrategy<K, V>: Send + Sync + 'static
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Process a cache write operation
    async fn process_write(&self, key: K, value: V, cache: &impl CacheStrategy<K, V>) -> Result<()>;
    
    /// Process a cache invalidation
    async fn process_invalidation(&self, key: &K, cache: &impl CacheStrategy<K, V>) -> Result<()>;
    
    /// Process a cache clear operation
    async fn process_clear(&self, cache: &impl CacheStrategy<K, V>) -> Result<()>;
    
    /// Get the name of the consistency strategy
    fn name(&self) -> &str;
    
    /// Get the configuration of the consistency strategy
    fn config(&self) -> ConsistencyConfig;
}

/// Cache consistency configuration
#[derive(Debug, Clone)]
pub struct ConsistencyConfig {
    /// Whether to propagate changes to other caches
    pub propagate_changes: bool,
    /// Maximum time to wait for propagation
    pub propagation_timeout: Option<Duration>,
    /// Whether to use synchronous or asynchronous propagation
    pub synchronous_propagation: bool,
    /// Maximum number of pending operations
    pub max_pending_operations: usize,
    /// Whether to retry failed operations
    pub retry_failed_operations: bool,
    /// Maximum number of retry attempts
    pub max_retry_attempts: usize,
    /// Retry delay
    pub retry_delay: Duration,
    /// Whether to log consistency operations
    pub log_operations: bool,
}

impl Default for ConsistencyConfig {
    fn default() -> Self {
        Self {
            propagate_changes: true,
            propagation_timeout: Some(Duration::from_secs(5)),
            synchronous_propagation: false,
            max_pending_operations: 1000,
            retry_failed_operations: true,
            max_retry_attempts: 3,
            retry_delay: Duration::from_millis(100),
            log_operations: true,
        }
    }
}

/// Cache operation type for consistency tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheOperation<K, V> {
    /// Write operation
    Write(K, V),
    /// Invalidation operation
    Invalidate(K),
    /// Clear operation
    Clear,
}

/// Write-through cache consistency strategy
///
/// This strategy immediately writes changes to the underlying data source
/// and propagates them to other caches. It provides strong consistency
/// but may have higher latency for write operations.
pub struct WriteThroughStrategy<K, V, D>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    D: DataSource<K, V>,
{
    /// Consistency configuration
    config: ConsistencyConfig,
    /// Data source for persistence
    data_source: Arc<D>,
    /// Change propagator for distributing changes
    propagator: Option<Arc<ChangePropagator<K, V>>>,
    /// Phantom data for K and V
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, D> WriteThroughStrategy<K, V, D>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    D: DataSource<K, V>,
{
    /// Create a new write-through strategy
    pub fn new(
        data_source: D,
        config: ConsistencyConfig,
        propagator: Option<Arc<ChangePropagator<K, V>>>,
    ) -> Self {
        Self {
            config,
            data_source: Arc::new(data_source),
            propagator,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<K, V, D> CacheConsistencyStrategy<K, V> for WriteThroughStrategy<K, V, D>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    D: DataSource<K, V>,
{
    #[instrument(skip(self, value, cache))]
    async fn process_write(&self, key: K, value: V, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Write-through: Processing write for key {:?}", key);
        }
        
        // First, write to the data source
        self.data_source.write(&key, &value).await?;
        
        // Then, update the cache
        cache.put(key.clone(), value.clone()).await?;
        
        // Finally, propagate the change if configured
        if self.config.propagate_changes {
            if let Some(propagator) = &self.propagator {
                let op = CacheOperation::Write(key, value);
                
                if self.config.synchronous_propagation {
                    // Wait for propagation to complete
                    propagator.propagate_and_wait(op, self.config.propagation_timeout).await?;
                } else {
                    // Fire and forget
                    propagator.propagate(op).await?;
                }
            }
        }
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_invalidation(&self, key: &K, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Write-through: Processing invalidation for key {:?}", key);
        }
        
        // First, remove from the data source
        self.data_source.delete(key).await?;
        
        // Then, remove from the cache
        cache.remove(key).await?;
        
        // Finally, propagate the invalidation if configured
        if self.config.propagate_changes {
            if let Some(propagator) = &self.propagator {
                let op = CacheOperation::Invalidate(key.clone());
                
                if self.config.synchronous_propagation {
                    // Wait for propagation to complete
                    propagator.propagate_and_wait(op, self.config.propagation_timeout).await?;
                } else {
                    // Fire and forget
                    propagator.propagate(op).await?;
                }
            }
        }
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_clear(&self, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Write-through: Processing clear");
        }
        
        // First, clear the data source
        self.data_source.clear().await?;
        
        // Then, clear the cache
        cache.clear().await?;
        
        // Finally, propagate the clear if configured
        if self.config.propagate_changes {
            if let Some(propagator) = &self.propagator {
                let op = CacheOperation::Clear;
                
                if self.config.synchronous_propagation {
                    // Wait for propagation to complete
                    propagator.propagate_and_wait(op, self.config.propagation_timeout).await?;
                } else {
                    // Fire and forget
                    propagator.propagate(op).await?;
                }
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "write-through"
    }
    
    fn config(&self) -> ConsistencyConfig {
        self.config.clone()
    }
}

/// Write-behind cache consistency strategy
///
/// This strategy immediately updates the cache and queues changes for
/// asynchronous writing to the underlying data source. It provides
/// better write performance but may have eventual consistency.
pub struct WriteBehindStrategy<K, V, D>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    D: DataSource<K, V>,
{
    /// Consistency configuration
    config: ConsistencyConfig,
    /// Data source for persistence
    data_source: Arc<D>,
    /// Change propagator for distributing changes
    propagator: Option<Arc<ChangePropagator<K, V>>>,
    /// Queue of pending operations
    pending_operations: Arc<RwLock<VecDeque<CacheOperation<K, V>>>>,
    /// Task handle for the background writer
    writer_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Phantom data for K and V
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, D> WriteBehindStrategy<K, V, D>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    D: DataSource<K, V>,
{
    /// Create a new write-behind strategy
    pub fn new(
        data_source: D,
        config: ConsistencyConfig,
        propagator: Option<Arc<ChangePropagator<K, V>>>,
        flush_interval: Duration,
    ) -> Self {
        let pending_operations = Arc::new(RwLock::new(VecDeque::new()));
        let writer_task = Arc::new(RwLock::new(None));
        
        let strategy = Self {
            config,
            data_source: Arc::new(data_source),
            propagator,
            pending_operations,
            writer_task,
            _phantom: PhantomData,
        };
        
        // Start the background writer task
        strategy.start_background_writer(flush_interval);
        
        strategy
    }
    
    /// Start the background writer task
    fn start_background_writer(&self, flush_interval: Duration) {
        let data_source = self.data_source.clone();
        let pending_operations = self.pending_operations.clone();
        let config = self.config.clone();
        let propagator = self.propagator.clone();
        
        // Spawn a background task to process the queue
        let task = tokio::spawn(async move {
            let mut interval = time::interval(flush_interval);
            
            loop {
                interval.tick().await;
                
                // Process pending operations
                let operations = {
                    let mut queue = pending_operations.write().await;
                    let ops: Vec<_> = queue.drain(..).collect();
                    ops
                };
                
                if operations.is_empty() {
                    continue;
                }
                
                if config.log_operations {
                    debug!("Write-behind: Processing {} pending operations", operations.len());
                }
                
                // Process each operation
                for op in operations {
                    let result = match &op {
                        CacheOperation::Write(key, value) => {
                            data_source.write(key, value).await
                        }
                        CacheOperation::Invalidate(key) => {
                            data_source.delete(key).await
                        }
                        CacheOperation::Clear => {
                            data_source.clear().await
                        }
                    };
                    
                    // Handle errors
                    if let Err(e) = result {
                        error!("Write-behind: Error processing operation: {:?}", e);
                        
                        // Retry if configured
                        if config.retry_failed_operations {
                            let mut queue = pending_operations.write().await;
                            queue.push_back(op);
                        }
                    } else if config.propagate_changes {
                        // Propagate the change if configured
                        if let Some(propagator) = &propagator {
                            if let Err(e) = propagator.propagate(op.clone()).await {
                                error!("Write-behind: Error propagating change: {:?}", e);
                            }
                        }
                    }
                }
            }
        });
        
        // Store the task handle
        tokio::spawn(async move {
            let mut writer_task = self.writer_task.write().await;
            *writer_task = Some(task);
        });
    }
    
    /// Stop the background writer task
    pub async fn stop_background_writer(&self) -> Result<()> {
        let mut writer_task = self.writer_task.write().await;
        
        if let Some(task) = writer_task.take() {
            task.abort();
            debug!("Write-behind: Stopped background writer task");
        }
        
        Ok(())
    }
    
    /// Flush pending operations
    pub async fn flush(&self) -> Result<()> {
        let operations = {
            let mut queue = self.pending_operations.write().await;
            let ops: Vec<_> = queue.drain(..).collect();
            ops
        };
        
        if operations.is_empty() {
            return Ok(());
        }
        
        if self.config.log_operations {
            debug!("Write-behind: Flushing {} pending operations", operations.len());
        }
        
        // Process each operation
        for op in operations {
            match &op {
                CacheOperation::Write(key, value) => {
                    self.data_source.write(key, value).await?;
                }
                CacheOperation::Invalidate(key) => {
                    self.data_source.delete(key).await?;
                }
                CacheOperation::Clear => {
                    self.data_source.clear().await?;
                }
            }
            
            // Propagate the change if configured
            if self.config.propagate_changes {
                if let Some(propagator) = &self.propagator {
                    propagator.propagate(op).await?;
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl<K, V, D> CacheConsistencyStrategy<K, V> for WriteBehindStrategy<K, V, D>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    D: DataSource<K, V>,
{
    #[instrument(skip(self, value, cache))]
    async fn process_write(&self, key: K, value: V, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Write-behind: Processing write for key {:?}", key);
        }
        
        // First, update the cache
        cache.put(key.clone(), value.clone()).await?;
        
        // Then, queue the operation for background processing
        let mut queue = self.pending_operations.write().await;
        
        // Check if we've reached the maximum number of pending operations
        if queue.len() >= self.config.max_pending_operations {
            warn!("Write-behind: Maximum number of pending operations reached");
            return Err(crate::error::AppError::resource_limit_exceeded(
                "Maximum number of pending cache operations reached"
            ));
        }
        
        queue.push_back(CacheOperation::Write(key, value));
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_invalidation(&self, key: &K, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Write-behind: Processing invalidation for key {:?}", key);
        }
        
        // First, remove from the cache
        cache.remove(key).await?;
        
        // Then, queue the operation for background processing
        let mut queue = self.pending_operations.write().await;
        
        // Check if we've reached the maximum number of pending operations
        if queue.len() >= self.config.max_pending_operations {
            warn!("Write-behind: Maximum number of pending operations reached");
            return Err(crate::error::AppError::resource_limit_exceeded(
                "Maximum number of pending cache operations reached"
            ));
        }
        
        queue.push_back(CacheOperation::Invalidate(key.clone()));
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_clear(&self, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Write-behind: Processing clear");
        }
        
        // First, clear the cache
        cache.clear().await?;
        
        // Then, queue the operation for background processing
        let mut queue = self.pending_operations.write().await;
        
        // Check if we've reached the maximum number of pending operations
        if queue.len() >= self.config.max_pending_operations {
            warn!("Write-behind: Maximum number of pending operations reached");
            return Err(crate::error::AppError::resource_limit_exceeded(
                "Maximum number of pending cache operations reached"
            ));
        }
        
        queue.push_back(CacheOperation::Clear);
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "write-behind"
    }
    
    fn config(&self) -> ConsistencyConfig {
        self.config.clone()
    }
}

/// Invalidation-based cache consistency strategy
///
/// This strategy focuses on invalidating cache entries when they change
/// rather than updating them. It's useful for read-heavy workloads where
/// keeping the cache in sync with writes is less important.
pub struct InvalidationStrategy<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Consistency configuration
    config: ConsistencyConfig,
    /// Change propagator for distributing changes
    propagator: Option<Arc<ChangePropagator<K, V>>>,
    /// Phantom data for K and V
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> InvalidationStrategy<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new invalidation strategy
    pub fn new(
        config: ConsistencyConfig,
        propagator: Option<Arc<ChangePropagator<K, V>>>,
    ) -> Self {
        Self {
            config,
            propagator,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<K, V> CacheConsistencyStrategy<K, V> for InvalidationStrategy<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, value, cache))]
    async fn process_write(&self, key: K, value: V, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Invalidation: Processing write for key {:?}", key);
        }
        
        // For invalidation strategy, we don't update the cache on write
        // Instead, we just propagate the invalidation
        
        if self.config.propagate_changes {
            if let Some(propagator) = &self.propagator {
                let op = CacheOperation::Invalidate(key);
                
                if self.config.synchronous_propagation {
                    // Wait for propagation to complete
                    propagator.propagate_and_wait(op, self.config.propagation_timeout).await?;
                } else {
                    // Fire and forget
                    propagator.propagate(op).await?;
                }
            }
        }
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_invalidation(&self, key: &K, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Invalidation: Processing invalidation for key {:?}", key);
        }
        
        // Remove from the cache
        cache.remove(key).await?;
        
        // Propagate the invalidation if configured
        if self.config.propagate_changes {
            if let Some(propagator) = &self.propagator {
                let op = CacheOperation::Invalidate(key.clone());
                
                if self.config.synchronous_propagation {
                    // Wait for propagation to complete
                    propagator.propagate_and_wait(op, self.config.propagation_timeout).await?;
                } else {
                    // Fire and forget
                    propagator.propagate(op).await?;
                }
            }
        }
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_clear(&self, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Invalidation: Processing clear");
        }
        
        // Clear the cache
        cache.clear().await?;
        
        // Propagate the clear if configured
        if self.config.propagate_changes {
            if let Some(propagator) = &self.propagator {
                let op = CacheOperation::Clear;
                
                if self.config.synchronous_propagation {
                    // Wait for propagation to complete
                    propagator.propagate_and_wait(op, self.config.propagation_timeout).await?;
                } else {
                    // Fire and forget
                    propagator.propagate(op).await?;
                }
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "invalidation"
    }
    
    fn config(&self) -> ConsistencyConfig {
        self.config.clone()
    }
}

/// Time-based expiration cache consistency strategy
///
/// This strategy relies on time-based expiration of cache entries to
/// ensure eventual consistency. It's simple but may lead to stale data.
pub struct ExpirationStrategy<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Consistency configuration
    config: ConsistencyConfig,
    /// Time-to-live for cache entries
    ttl: Duration,
    /// Phantom data for K and V
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> ExpirationStrategy<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new expiration strategy
    pub fn new(
        config: ConsistencyConfig,
        ttl: Duration,
    ) -> Self {
        Self {
            config,
            ttl,
            _phantom: PhantomData,
        }
    }
    
    /// Get the time-to-live
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

#[async_trait]
impl<K, V> CacheConsistencyStrategy<K, V> for ExpirationStrategy<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, value, cache))]
    async fn process_write(&self, key: K, value: V, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Expiration: Processing write for key {:?}", key);
        }
        
        // Update the cache with the new value
        // The cache implementation should handle TTL
        cache.put(key, value).await?;
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_invalidation(&self, key: &K, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Expiration: Processing invalidation for key {:?}", key);
        }
        
        // Remove from the cache
        cache.remove(key).await?;
        
        Ok(())
    }
    
    #[instrument(skip(self, cache))]
    async fn process_clear(&self, cache: &impl CacheStrategy<K, V>) -> Result<()> {
        if self.config.log_operations {
            debug!("Expiration: Processing clear");
        }
        
        // Clear the cache
        cache.clear().await?;
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "expiration"
    }
    
    fn config(&self) -> ConsistencyConfig {
        self.config.clone()
    }
}

/// Data source trait for cache consistency strategies
#[async_trait]
pub trait DataSource<K, V>: Send + Sync + 'static
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Read a value from the data source
    async fn read(&self, key: &K) -> Result<Option<V>>;
    
    /// Write a value to the data source
    async fn write(&self, key: &K, value: &V) -> Result<()>;
    
    /// Delete a value from the data source
    async fn delete(&self, key: &K) -> Result<()>;
    
    /// Clear all values from the data source
    async fn clear(&self) -> Result<()>;
}

/// Repository data source adapter
///
/// This adapter allows a repository to be used as a data source
/// for cache consistency strategies.
pub struct RepositoryDataSource<R, K, V, F>
where
    R: crate::repositories::base::Repository<V, F>,
    K: CacheKey,
    V: Clone + Send + Sync + Serialize + DeserializeOwned + Debug + 'static,
    F: Send + Sync + Debug + 'static,
{
    /// The repository
    repository: R,
    /// Default filter for repository operations
    default_filter: F,
    /// Key extractor function
    key_extractor: Box<dyn Fn(&V) -> K + Send + Sync>,
    /// Phantom data for K
    _phantom: PhantomData<K>,
}

impl<R, K, V, F> RepositoryDataSource<R, K, V, F>
where
    R: crate::repositories::base::Repository<V, F>,
    K: CacheKey,
    V: Clone + Send + Sync + Serialize + DeserializeOwned + Debug + 'static,
    F: Send + Sync + Debug + 'static,
{
    /// Create a new repository data source adapter
    pub fn new(
        repository: R,
        default_filter: F,
        key_extractor: impl Fn(&V) -> K + Send + Sync + 'static,
    ) -> Self {
        Self {
            repository,
            default_filter,
            key_extractor: Box::new(key_extractor),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<R, K, V, F> DataSource<K, V> for RepositoryDataSource<R, K, V, F>
where
    R: crate::repositories::base::Repository<V, F>,
    K: CacheKey,
    V: Clone + Send + Sync + Serialize + DeserializeOwned + Debug + 'static,
    F: Send + Sync + Debug + Clone + 'static,
{
    async fn read(&self, key: &K) -> Result<Option<V>> {
        // This is a simplified implementation that assumes the key is a UUID
        // In a real implementation, you would need to convert the key to the appropriate type
        // based on the repository's requirements
        if let Some(uuid) = key_to_uuid(key) {
            self.repository.get_by_id(&uuid).await
        } else {
            Err(crate::error::AppError::validation(
                "Invalid key type for repository data source"
            ))
        }
    }
    
    async fn write(&self, key: &K, value: &V) -> Result<()> {
        // Ensure the key matches the entity
        let entity_key = (self.key_extractor)(value);
        if &entity_key != key {
            return Err(crate::error::AppError::validation(
                "Key mismatch between provided key and entity key"
            ));
        }
        
        self.repository.update(value).await
    }
    
    async fn delete(&self, key: &K) -> Result<()> {
        if let Some(uuid) = key_to_uuid(key) {
            self.repository.delete(&uuid).await
        } else {
            Err(crate::error::AppError::validation(
                "Invalid key type for repository data source"
            ))
        }
    }
    
    async fn clear(&self) -> Result<()> {
        // This is a simplified implementation that doesn't actually clear the repository
        // In a real implementation, you would need to implement a clear method on the repository
        warn!("Repository data source clear operation not implemented");
        Ok(())
    }
}

/// Helper function to convert a cache key to a UUID
fn key_to_uuid<K: CacheKey>(key: &K) -> Option<Uuid> {
    // This is a simplified implementation that only handles UUID keys
    // In a real implementation, you would need to handle different key types
    if let Some(uuid) = key.type_id().downcast_ref::<Uuid>() {
        Some(*uuid)
    } else {
        None
    }
}

/// Change propagator for distributing cache changes
pub struct ChangePropagator<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Broadcast channel for propagating changes
    tx: broadcast::Sender<CacheOperation<K, V>>,
    /// Receiver for the broadcast channel
    rx: Arc<RwLock<broadcast::Receiver<CacheOperation<K, V>>>>,
    /// Registered caches
    caches: Arc<RwLock<HashMap<String, Arc<dyn CacheStrategy<K, V>>>>>,
    /// Registered consistency strategies
    strategies: Arc<RwLock<HashMap<String, Arc<dyn CacheConsistencyStrategy<K, V>>>>>,
}

impl<K, V> ChangePropagator<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new change propagator
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = broadcast::channel(capacity);
        
        Self {
            tx,
            rx: Arc::new(RwLock::new(rx)),
            caches: Arc::new(RwLock::new(HashMap::new())),
            strategies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a cache
    pub async fn register_cache(&self, name: impl Into<String>, cache: Arc<dyn CacheStrategy<K, V>>) {
        let mut caches = self.caches.write().await;
        caches.insert(name.into(), cache);
    }
    
    /// Unregister a cache
    pub async fn unregister_cache(&self, name: &str) {
        let mut caches = self.caches.write().await;
        caches.remove(name);
    }
    
    /// Register a consistency strategy
    pub async fn register_strategy(&self, name: impl Into<String>, strategy: Arc<dyn CacheConsistencyStrategy<K, V>>) {
        let mut strategies = self.strategies.write().await;
        strategies.insert(name.into(), strategy);
    }
    
    /// Unregister a consistency strategy
    pub async fn unregister_strategy(&self, name: &str) {
        let mut strategies = self.strategies.write().await;
        strategies.remove(name);
    }
    
    /// Propagate a change
    pub async fn propagate(&self, operation: CacheOperation<K, V>) -> Result<()> {
        // Send the operation to all subscribers
        if let Err(e) = self.tx.send(operation) {
            warn!("Failed to propagate cache operation: {}", e);
        }
        
        Ok(())
    }
    
    /// Propagate a change and wait for it to be processed
    pub async fn propagate_and_wait(&self, operation: CacheOperation<K, V>, timeout: Option<Duration>) -> Result<()> {
        // Create a oneshot channel for the response
        let (tx, rx) = oneshot::channel();
        
        // Send the operation with the response channel
        if let Err(e) = self.tx.send(operation) {
            warn!("Failed to propagate cache operation: {}", e);
            return Err(crate::error::AppError::internal(
                format!("Failed to propagate cache operation: {}", e)
            ));
        }
        
        // Wait for the response with timeout
        if let Some(timeout) = timeout {
            tokio::select! {
                result = rx => {
                    match result {
                        Ok(()) => Ok(()),
                        Err(_) => Err(crate::error::AppError::internal(
                            "Failed to receive propagation confirmation"
                        )),
                    }
                }
                _ = tokio::time::sleep(timeout) => {
                    Err(crate::error::AppError::internal(
                        "Timeout waiting for propagation confirmation"
                    ))
                }
            }
        } else {
            // Wait indefinitely
            match rx.await {
                Ok(()) => Ok(()),
                Err(_) => Err(crate::error::AppError::internal(
                    "Failed to receive propagation confirmation"
                )),
            }
        }
    }
    
    /// Start listening for changes
    pub async fn start_listening(&self) -> Result<JoinHandle<()>> {
        let rx = self.rx.write().await.resubscribe();
        let caches = self.caches.clone();
        let strategies = self.strategies.clone();
        
        // Spawn a task to listen for changes
        let task = tokio::spawn(async move {
            let mut rx = rx;
            
            loop {
                match rx.recv().await {
                    Ok(operation) => {
                        // Process the operation for each cache
                        let caches_guard = caches.read().await;
                        let strategies_guard = strategies.read().await;
                        
                        for (cache_name, cache) in caches_guard.iter() {
                            // Find a strategy for this cache
                            if let Some(strategy) = strategies_guard.get(cache_name) {
                                match &operation {
                                    CacheOperation::Write(key, value) => {
                                        if let Err(e) = strategy.process_write(key.clone(), value.clone(), cache.as_ref()).await {
                                            error!("Error processing write for cache {}: {:?}", cache_name, e);
                                        }
                                    }
                                    CacheOperation::Invalidate(key) => {
                                        if let Err(e) = strategy.process_invalidation(key, cache.as_ref()).await {
                                            error!("Error processing invalidation for cache {}: {:?}", cache_name, e);
                                        }
                                    }
                                    CacheOperation::Clear => {
                                        if let Err(e) = strategy.process_clear(cache.as_ref()).await {
                                            error!("Error processing clear for cache {}: {:?}", cache_name, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving cache operation: {:?}", e);
                        
                        // If it's a lagged error, resubscribe
                        if e.is_lagged() {
                            warn!("Resubscribing to cache operations due to lag");
                            rx = broadcast::Receiver::resubscribe(&rx);
                        }
                    }
                }
            }
        });
        
        Ok(task)
    }
}

/// Consistent cache that applies a consistency strategy to a cache
pub struct ConsistentCache<K, V, C, S>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    C: CacheStrategy<K, V>,
    S: CacheConsistencyStrategy<K, V>,
{
    /// The underlying cache
    cache: C,
    /// The consistency strategy
    strategy: S,
    /// Phantom data for K and V
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, C, S> ConsistentCache<K, V, C, S>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    C: CacheStrategy<K, V>,
    S: CacheConsistencyStrategy<K, V>,
{
    /// Create a new consistent cache
    pub fn new(cache: C, strategy: S) -> Self {
        Self {
            cache,
            strategy,
            _phantom: PhantomData,
        }
    }
    
    /// Get the underlying cache
    pub fn cache(&self) -> &C {
        &self.cache
    }
    
    /// Get the consistency strategy
    pub fn strategy(&self) -> &S {
        &self.strategy
    }
}

#[async_trait]
impl<K, V, C, S> CacheStrategy<K, V> for ConsistentCache<K, V, C, S>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
    C: CacheStrategy<K, V>,
    S: CacheConsistencyStrategy<K, V>,
{
    #[instrument(skip(self))]
    async fn get(&self, key: &K) -> Option<V> {
        // For get operations, we just delegate to the underlying cache
        self.cache.get(key).await
    }
    
    #[instrument(skip(self, value))]
    async fn put(&self, key: K, value: V) -> Result<()> {
        // For put operations, we use the consistency strategy
        self.strategy.process_write(key, value, &self.cache).await
    }
    
    #[instrument(skip(self))]
    async fn remove(&self, key: &K) -> Result<()> {
        // For remove operations, we use the consistency strategy
        self.strategy.process_invalidation(key, &self.cache).await
    }
    
    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        // For clear operations, we use the consistency strategy
        self.strategy.process_clear(&self.cache).await
    }
    
    async fn len(&self) -> usize {
        // For len operations, we just delegate to the underlying cache
        self.cache.len().await
    }
}

/// Cache consistency factory for creating different types of consistent caches
pub struct CacheConsistencyFactory;

impl CacheConsistencyFactory {
    /// Create a write-through consistent cache
    pub fn write_through<K, V, D>(
        cache: impl CacheStrategy<K, V>,
        data_source: D,
        config: Option<ConsistencyConfig>,
        propagator: Option<Arc<ChangePropagator<K, V>>>,
    ) -> impl CacheStrategy<K, V>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
        D: DataSource<K, V>,
    {
        let strategy = WriteThroughStrategy::new(
            data_source,
            config.unwrap_or_default(),
            propagator,
        );
        
        ConsistentCache::new(cache, strategy)
    }
    
    /// Create a write-behind consistent cache
    pub fn write_behind<K, V, D>(
        cache: impl CacheStrategy<K, V>,
        data_source: D,
        config: Option<ConsistencyConfig>,
        propagator: Option<Arc<ChangePropagator<K, V>>>,
        flush_interval: Option<Duration>,
    ) -> impl CacheStrategy<K, V>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
        D: DataSource<K, V>,
    {
        let strategy = WriteBehindStrategy::new(
            data_source,
            config.unwrap_or_default(),
            propagator,
            flush_interval.unwrap_or_else(|| Duration::from_secs(5)),
        );
        
        ConsistentCache::new(cache, strategy)
    }
    
    /// Create an invalidation-based consistent cache
    pub fn invalidation<K, V>(
        cache: impl CacheStrategy<K, V>,
        config: Option<ConsistencyConfig>,
        propagator: Option<Arc<ChangePropagator<K, V>>>,
    ) -> impl CacheStrategy<K, V>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
    {
        let strategy = InvalidationStrategy::new(
            config.unwrap_or_default(),
            propagator,
        );
        
        ConsistentCache::new(cache, strategy)
    }
    
    /// Create a time-based expiration consistent cache
    pub fn expiration<K, V>(
        cache: impl CacheStrategy<K, V>,
        config: Option<ConsistencyConfig>,
        ttl: Duration,
    ) -> impl CacheStrategy<K, V>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
    {
        let strategy = ExpirationStrategy::new(
            config.unwrap_or_default(),
            ttl,
        );
        
        ConsistentCache::new(cache, strategy)
    }
    
    /// Create a change propagator
    pub fn change_propagator<K, V>(capacity: usize) -> Arc<ChangePropagator<K, V>>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
    {
        Arc::new(ChangePropagator::new(capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    // Mock data source for testing
    struct MockDataSource<K, V> {
        data: Arc<RwLock<HashMap<K, V>>>,
        write_count: Arc<AtomicUsize>,
        delete_count: Arc<AtomicUsize>,
        clear_count: Arc<AtomicUsize>,
    }
    
    impl<K, V> MockDataSource<K, V>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
    {
        fn new() -> Self {
            Self {
                data: Arc::new(RwLock::new(HashMap::new())),
                write_count: Arc::new(AtomicUsize::new(0)),
                delete_count: Arc::new(AtomicUsize::new(0)),
                clear_count: Arc::new(AtomicUsize::new(0)),
            }
        }
        
        fn get_write_count(&self) -> usize {
            self.write_count.load(Ordering::SeqCst)
        }
        
        fn get_delete_count(&self) -> usize {
            self.delete_count.load(Ordering::SeqCst)
        }
        
        fn get_clear_count(&self) -> usize {
            self.clear_count.load(Ordering::SeqCst)
        }
    }
    
    #[async_trait]
    impl<K, V> DataSource<K, V> for MockDataSource<K, V>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
    {
        async fn read(&self, key: &K) -> Result<Option<V>> {
            let data = self.data.read().await;
            Ok(data.get(key).cloned())
        }
        
        async fn write(&self, key: &K, value: &V) -> Result<()> {
            let mut data = self.data.write().await;
            data.insert(key.clone(), value.clone());
            self.write_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        
        async fn delete(&self, key: &K) -> Result<()> {
            let mut data = self.data.write().await;
            data.remove(key);
            self.delete_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        
        async fn clear(&self) -> Result<()> {
            let mut data = self.data.write().await;
            data.clear();
            self.clear_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_write_through_strategy() -> Result<()> {
        // Create a mock data source
        let data_source = MockDataSource::<String, String>::new();
        
        // Create a memory cache
        let cache = crate::repositories::cache::CacheFactory::memory_cache(100, None);
        
        // Create a write-through strategy
        let strategy = WriteThroughStrategy::new(
            data_source.clone(),
            ConsistencyConfig::default(),
            None,
        );
        
        // Create a consistent cache
        let consistent_cache = ConsistentCache::new(cache, strategy);
        
        // Put a value
        consistent_cache.put("key1".to_string(), "value1".to_string()).await?;
        
        // Verify the value is in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Verify the value is in the data source
        let value = data_source.read(&"key1".to_string()).await?;
        assert_eq!(value, Some("value1".to_string()));
        
        // Verify the write count
        assert_eq!(data_source.get_write_count(), 1);
        
        // Remove the value
        consistent_cache.remove(&"key1".to_string()).await?;
        
        // Verify the value is not in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
        
        // Verify the value is not in the data source
        let value = data_source.read(&"key1".to_string()).await?;
        assert_eq!(value, None);
        
        // Verify the delete count
        assert_eq!(data_source.get_delete_count(), 1);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_write_behind_strategy() -> Result<()> {
        // Create a mock data source
        let data_source = MockDataSource::<String, String>::new();
        
        // Create a memory cache
        let cache = crate::repositories::cache::CacheFactory::memory_cache(100, None);
        
        // Create a write-behind strategy
        let strategy = WriteBehindStrategy::new(
            data_source.clone(),
            ConsistencyConfig::default(),
            None,
            Duration::from_millis(100), // Short flush interval for testing
        );
        
        // Create a consistent cache
        let consistent_cache = ConsistentCache::new(cache, strategy);
        
        // Put a value
        consistent_cache.put("key1".to_string(), "value1".to_string()).await?;
        
        // Verify the value is in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Wait for the background writer to flush
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Verify the value is in the data source
        let value = data_source.read(&"key1".to_string()).await?;
        assert_eq!(value, Some("value1".to_string()));
        
        // Verify the write count
        assert_eq!(data_source.get_write_count(), 1);
        
        // Remove the value
        consistent_cache.remove(&"key1".to_string()).await?;
        
        // Verify the value is not in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
        
        // Wait for the background writer to flush
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Verify the value is not in the data source
        let value = data_source.read(&"key1".to_string()).await?;
        assert_eq!(value, None);
        
        // Verify the delete count
        assert_eq!(data_source.get_delete_count(), 1);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_invalidation_strategy() -> Result<()> {
        // Create a memory cache
        let cache = crate::repositories::cache::CacheFactory::memory_cache(100, None);
        
        // Create an invalidation strategy
        let strategy = InvalidationStrategy::new(
            ConsistencyConfig::default(),
            None,
        );
        
        // Create a consistent cache
        let consistent_cache = ConsistentCache::new(cache, strategy);
        
        // Put a value directly in the cache
        consistent_cache.cache().put("key1".to_string(), "value1".to_string()).await?;
        
        // Verify the value is in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Process a write (which should invalidate the key)
        consistent_cache.put("key1".to_string(), "value2".to_string()).await?;
        
        // Verify the value is no longer in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_expiration_strategy() -> Result<()> {
        // Create a memory cache with TTL
        let cache = crate::repositories::cache::CacheFactory::memory_cache(100, Some(Duration::from_millis(200)));
        
        // Create an expiration strategy
        let strategy = ExpirationStrategy::new(
            ConsistencyConfig::default(),
            Duration::from_millis(200),
        );
        
        // Create a consistent cache
        let consistent_cache = ConsistentCache::new(cache, strategy);
        
        // Put a value
        consistent_cache.put("key1".to_string(), "value1".to_string()).await?;
        
        // Verify the value is in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Wait for the TTL to expire
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        // Verify the value is no longer in the cache
        let value = consistent_cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_change_propagator() -> Result<()> {
        // Create a change propagator
        let propagator = CacheConsistencyFactory::change_propagator::<String, String>(100);
        
        // Create two memory caches
        let cache1 = crate::repositories::cache::CacheFactory::memory_cache(100, None);
        let cache2 = crate::repositories::cache::CacheFactory::memory_cache(100, None);
        
        // Create two invalidation strategies
        let strategy1 = InvalidationStrategy::new(
            ConsistencyConfig::default(),
            Some(propagator.clone()),
        );
        let strategy2 = InvalidationStrategy::new(
            ConsistencyConfig::default(),
            Some(propagator.clone()),
        );
        
        // Create two consistent caches
        let consistent_cache1 = ConsistentCache::new(cache1.clone(), strategy1);
        let consistent_cache2 = ConsistentCache::new(cache2.clone(), strategy2);
        
        // Register the caches with the propagator
        propagator.register_cache("cache1", Arc::new(cache1)).await;
        propagator.register_cache("cache2", Arc::new(cache2)).await;
        
        // Register the strategies with the propagator
        propagator.register_strategy("strategy1", Arc::new(strategy1)).await;
        propagator.register_strategy("strategy2", Arc::new(strategy2)).await;
        
        // Start listening for changes
        let _listener = propagator.start_listening().await?;
        
        // Put a value in cache1
        consistent_cache1.cache().put("key1".to_string(), "value1".to_string()).await?;
        consistent_cache2.cache().put("key1".to_string(), "value1".to_string()).await?;
        
        // Verify the value is in both caches
        let value1 = consistent_cache1.get(&"key1".to_string()).await;
        let value2 = consistent_cache2.get(&"key1".to_string()).await;
        assert_eq!(value1, Some("value1".to_string()));
        assert_eq!(value2, Some("value1".to_string()));
        
        // Invalidate the key in cache1
        consistent_cache1.remove(&"key1".to_string()).await?;
        
        // Wait for propagation
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Verify the value is not in either cache
        let value1 = consistent_cache1.get(&"key1".to_string()).await;
        let value2 = consistent_cache2.get(&"key1".to_string()).await;
        assert_eq!(value1, None);
        assert_eq!(value2, None);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_factory_methods() -> Result<()> {
        // Create a mock data source
        let data_source = MockDataSource::<String, String>::new();
        
        // Create a memory cache
        let cache = crate::repositories::cache::CacheFactory::memory_cache(100, None);
        
        // Create a write-through cache using the factory
        let write_through_cache = CacheConsistencyFactory::write_through(
            cache,
            data_source.clone(),
            None,
            None,
        );
        
        // Put a value
        write_through_cache.put("key1".to_string(), "value1".to_string()).await?;
        
        // Verify the value is in the cache
        let value = write_through_cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Verify the value is in the data source
        let value = data_source.read(&"key1".to_string()).await?;
        assert_eq!(value, Some("value1".to_string()));
        
        Ok(())
    }
}