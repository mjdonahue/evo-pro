//! Caching strategies for repositories
//!
//! This module provides caching strategies for repositories, including
//! memory, disk, and hybrid caching.

use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};
use tracing::{debug, error, instrument, warn};

use crate::error::Result;

/// Cache key trait for entities that can be cached
pub trait CacheKey: Hash + Eq + Clone + Debug + Send + Sync + 'static {}

// Implement CacheKey for common key types
impl CacheKey for uuid::Uuid {}
impl CacheKey for String {}
impl CacheKey for i32 {}
impl CacheKey for i64 {}
impl CacheKey for u32 {}
impl CacheKey for u64 {}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// The cached value
    value: V,
    /// When the entry was created
    created_at: Instant,
    /// When the entry was last accessed
    last_accessed: Instant,
    /// Number of times the entry has been accessed
    access_count: u64,
}

/// Cache strategy trait
#[async_trait]
pub trait CacheStrategy<K, V>: Send + Sync + 'static
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Get a value from the cache
    async fn get(&self, key: &K) -> Option<V>;
    
    /// Put a value in the cache
    async fn put(&self, key: K, value: V) -> Result<()>;
    
    /// Remove a value from the cache
    async fn remove(&self, key: &K) -> Result<()>;
    
    /// Clear the cache
    async fn clear(&self) -> Result<()>;
    
    /// Get the number of entries in the cache
    async fn len(&self) -> usize;
    
    /// Check if the cache is empty
    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

/// Memory cache strategy
pub struct MemoryCache<K, V> {
    /// The cache entries
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Maximum number of entries
    max_entries: usize,
    /// Time-to-live for entries
    ttl: Option<Duration>,
}

impl<K, V> MemoryCache<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new memory cache
    pub fn new(max_entries: usize, ttl: Option<Duration>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_entries,
            ttl,
        }
    }
    
    /// Evict expired entries
    async fn evict_expired(&self) {
        let mut entries = self.entries.write().await;
        
        if let Some(ttl) = self.ttl {
            let now = Instant::now();
            entries.retain(|_, entry| now.duration_since(entry.created_at) < ttl);
        }
    }
    
    /// Evict entries if the cache is full
    async fn evict_if_full(&self) {
        let mut entries = self.entries.write().await;
        
        if entries.len() >= self.max_entries {
            // Evict least recently used entries
            let mut entries_vec: Vec<_> = entries.iter().collect();
            entries_vec.sort_by_key(|(_, entry)| entry.last_accessed);
            
            // Remove the oldest 10% of entries
            let to_remove = (self.max_entries as f64 * 0.1).max(1.0) as usize;
            for (key, _) in entries_vec.iter().take(to_remove) {
                entries.remove(*key);
            }
        }
    }
}

#[async_trait]
impl<K, V> CacheStrategy<K, V> for MemoryCache<K, V>
where
    K: CacheKey,
    V: Clone + Send + Sync + 'static,
{
    #[instrument(skip(self))]
    async fn get(&self, key: &K) -> Option<V> {
        // Evict expired entries
        self.evict_expired().await;
        
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            // Update access metadata
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            
            Some(entry.value.clone())
        } else {
            None
        }
    }
    
    #[instrument(skip(self, value))]
    async fn put(&self, key: K, value: V) -> Result<()> {
        // Evict expired entries
        self.evict_expired().await;
        
        // Evict entries if the cache is full
        self.evict_if_full().await;
        
        let now = Instant::now();
        let entry = CacheEntry {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
        };
        
        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn remove(&self, key: &K) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.remove(key);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.clear();
        Ok(())
    }
    
    async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }
}

/// Disk cache strategy
pub struct DiskCache<K, V> {
    /// Cache directory
    cache_dir: PathBuf,
    /// Time-to-live for entries
    ttl: Option<Duration>,
    /// In-memory index of cached keys
    index: Arc<RwLock<HashMap<K, Instant>>>,
}

impl<K, V> DiskCache<K, V>
where
    K: CacheKey + Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    /// Create a new disk cache
    pub async fn new(cache_dir: PathBuf, ttl: Option<Duration>) -> Result<Self> {
        // Create the cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir).await?;
        
        // Initialize the index
        let index = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            cache_dir,
            ttl,
            index,
        })
    }
    
    /// Get the path for a key
    fn key_path(&self, key: &K) -> Result<PathBuf> {
        let key_bytes = bincode::serialize(key)?;
        let key_hash = format!("{:x}", md5::compute(&key_bytes));
        Ok(self.cache_dir.join(key_hash))
    }
    
    /// Evict expired entries
    async fn evict_expired(&self) -> Result<()> {
        if let Some(ttl) = self.ttl {
            let now = Instant::now();
            let mut index = self.index.write().await;
            
            // Collect keys to remove
            let keys_to_remove: Vec<_> = index
                .iter()
                .filter(|(_, created_at)| now.duration_since(**created_at) > ttl)
                .map(|(key, _)| key.clone())
                .collect();
            
            // Remove from index and disk
            for key in keys_to_remove {
                index.remove(&key);
                let path = self.key_path(&key)?;
                if let Err(e) = fs::remove_file(&path).await {
                    warn!("Failed to remove expired cache file: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl<K, V> CacheStrategy<K, V> for DiskCache<K, V>
where
    K: CacheKey + Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    #[instrument(skip(self))]
    async fn get(&self, key: &K) -> Option<V> {
        // Evict expired entries
        if let Err(e) = self.evict_expired().await {
            error!("Failed to evict expired entries: {}", e);
        }
        
        // Check if the key exists in the index
        let index = self.index.read().await;
        if !index.contains_key(key) {
            return None;
        }
        
        // Get the path for the key
        let path = match self.key_path(key) {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to get key path: {}", e);
                return None;
            }
        };
        
        // Read the file
        let mut file = match File::open(&path).await {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open cache file: {}", e);
                return None;
            }
        };
        
        // Read the contents
        let mut contents = Vec::new();
        if let Err(e) = file.read_to_end(&mut contents).await {
            error!("Failed to read cache file: {}", e);
            return None;
        }
        
        // Deserialize the value
        match bincode::deserialize(&contents) {
            Ok(value) => Some(value),
            Err(e) => {
                error!("Failed to deserialize cache value: {}", e);
                None
            }
        }
    }
    
    #[instrument(skip(self, value))]
    async fn put(&self, key: K, value: V) -> Result<()> {
        // Evict expired entries
        self.evict_expired().await?;
        
        // Get the path for the key
        let path = self.key_path(&key)?;
        
        // Serialize the value
        let bytes = bincode::serialize(&value)?;
        
        // Write to the file
        let mut file = File::create(&path).await?;
        file.write_all(&bytes).await?;
        
        // Update the index
        let mut index = self.index.write().await;
        index.insert(key, Instant::now());
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn remove(&self, key: &K) -> Result<()> {
        // Get the path for the key
        let path = self.key_path(key)?;
        
        // Remove from the index
        let mut index = self.index.write().await;
        index.remove(key);
        
        // Remove the file if it exists
        if path.exists() {
            fs::remove_file(path).await?;
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        // Clear the index
        let mut index = self.index.write().await;
        index.clear();
        
        // Remove all files in the cache directory
        let mut dir = fs::read_dir(&self.cache_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            fs::remove_file(entry.path()).await?;
        }
        
        Ok(())
    }
    
    async fn len(&self) -> usize {
        let index = self.index.read().await;
        index.len()
    }
}

/// Hybrid cache strategy that combines memory and disk caching
pub struct HybridCache<K, V> {
    /// Memory cache
    memory_cache: MemoryCache<K, V>,
    /// Disk cache
    disk_cache: DiskCache<K, V>,
}

impl<K, V> HybridCache<K, V>
where
    K: CacheKey + Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    /// Create a new hybrid cache
    pub async fn new(
        max_memory_entries: usize,
        memory_ttl: Option<Duration>,
        cache_dir: PathBuf,
        disk_ttl: Option<Duration>,
    ) -> Result<Self> {
        let memory_cache = MemoryCache::new(max_memory_entries, memory_ttl);
        let disk_cache = DiskCache::new(cache_dir, disk_ttl).await?;
        
        Ok(Self {
            memory_cache,
            disk_cache,
        })
    }
}

#[async_trait]
impl<K, V> CacheStrategy<K, V> for HybridCache<K, V>
where
    K: CacheKey + Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    #[instrument(skip(self))]
    async fn get(&self, key: &K) -> Option<V> {
        // Try to get from memory cache first
        if let Some(value) = self.memory_cache.get(key).await {
            debug!("Cache hit (memory): {:?}", key);
            return Some(value);
        }
        
        // If not in memory, try disk cache
        if let Some(value) = self.disk_cache.get(key).await {
            debug!("Cache hit (disk): {:?}", key);
            
            // Store in memory cache for future access
            if let Err(e) = self.memory_cache.put(key.clone(), value.clone()).await {
                warn!("Failed to store in memory cache: {}", e);
            }
            
            return Some(value);
        }
        
        debug!("Cache miss: {:?}", key);
        None
    }
    
    #[instrument(skip(self, value))]
    async fn put(&self, key: K, value: V) -> Result<()> {
        // Store in both memory and disk cache
        let key_clone = key.clone();
        let value_clone = value.clone();
        
        // Store in memory cache
        self.memory_cache.put(key, value).await?;
        
        // Store in disk cache
        self.disk_cache.put(key_clone, value_clone).await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn remove(&self, key: &K) -> Result<()> {
        // Remove from both memory and disk cache
        self.memory_cache.remove(key).await?;
        self.disk_cache.remove(key).await?;
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        // Clear both memory and disk cache
        self.memory_cache.clear().await?;
        self.disk_cache.clear().await?;
        
        Ok(())
    }
    
    async fn len(&self) -> usize {
        // Return the size of the disk cache, which should be more comprehensive
        self.disk_cache.len().await
    }
}

/// Cache factory for creating different types of caches
pub struct CacheFactory;

impl CacheFactory {
    /// Create a memory cache
    pub fn memory_cache<K, V>(
        max_entries: usize,
        ttl: Option<Duration>,
    ) -> impl CacheStrategy<K, V>
    where
        K: CacheKey,
        V: Clone + Send + Sync + 'static,
    {
        MemoryCache::new(max_entries, ttl)
    }
    
    /// Create a disk cache
    pub async fn disk_cache<K, V>(
        cache_dir: PathBuf,
        ttl: Option<Duration>,
    ) -> Result<impl CacheStrategy<K, V>>
    where
        K: CacheKey + Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    {
        DiskCache::new(cache_dir, ttl).await
    }
    
    /// Create a hybrid cache
    pub async fn hybrid_cache<K, V>(
        max_memory_entries: usize,
        memory_ttl: Option<Duration>,
        cache_dir: PathBuf,
        disk_ttl: Option<Duration>,
    ) -> Result<impl CacheStrategy<K, V>>
    where
        K: CacheKey + Serialize + DeserializeOwned,
        V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    {
        HybridCache::new(max_memory_entries, memory_ttl, cache_dir, disk_ttl).await
    }
}