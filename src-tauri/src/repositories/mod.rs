//! Repository pattern implementation for data access
//! 
//! This module provides a clean separation of concerns for data access operations
//! by implementing the repository pattern. Each entity has its own repository
//! that encapsulates database access logic.

pub mod base;
pub mod cache;
pub mod cache_metrics;
pub mod cache_warming;
pub mod factory;
pub mod task_repository;
pub mod query_builder;
pub mod validation;

#[cfg(test)]
pub mod tests;

// Re-export repositories and factory for easier access
pub use task_repository::TaskRepository;
pub use factory::RepositoryFactory;
pub use cache::{CacheFactory, CacheStrategy};
pub use cache_warming::{CacheWarmer, CacheWarmingFactory, DataLoader, WarmingConfig};
pub use cache_metrics::{CacheMetrics, CacheMonitor, MetricsCollector, MonitoredCacheFactory};
