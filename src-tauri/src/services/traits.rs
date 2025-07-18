use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use uuid::Uuid;

/// Context passed to all service operations
#[derive(Clone)]
pub struct ServiceContext {
    /// Database connection pool
    pub db: Pool<Sqlite>,
    /// Actor system reference for communication
    pub actor_system: Arc<ActorSystem>,
    /// Current user/agent context
    pub auth_context: Option<AuthContext>,
    /// Request metadata
    pub request_id: Uuid,
    /// Workspace context
    pub workspace_id: Option<Uuid>,
}

/// Authentication context for service operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub participant_id: Uuid,
    pub workspace_id: Uuid,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// Service operation result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResult<T> {
    pub data: T,
    pub metadata: Option<ServiceMetadata>,
}

/// Metadata attached to service operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub request_id: Uuid,
    pub execution_time_ms: u64,
    pub cache_hit: bool,
    pub actor_interactions: Vec<ActorInteraction>,
}

/// Record of actor interactions during service operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorInteraction {
    pub actor_type: String,
    pub operation: String,
    pub duration_ms: u64,
    pub success: bool,
}

/// Base trait for all services with CRUD operations
#[async_trait]
pub trait Service<T, CreateInput, UpdateInput, Filter> {
    /// Create a new entity
    async fn create(&self, ctx: &ServiceContext, input: CreateInput) -> Result<ServiceResult<T>>;

    /// Get entity by ID
    async fn get(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<Option<T>>>;

    /// Update existing entity
    async fn update(&self, ctx: &ServiceContext, entity: UpdateInput) -> Result<ServiceResult<T>>;

    /// Delete entity by ID
    async fn delete(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<()>>;

    /// List entities with filtering and pagination
    async fn list(&self, ctx: &ServiceContext, filter: Filter) -> Result<ServiceResult<Vec<T>>>;

    /// Count entities matching filter
    async fn count(&self, ctx: &ServiceContext, filter: Filter) -> Result<ServiceResult<i64>>;
}

/// Trait for services that support real-time events
#[async_trait]
pub trait EventEmitter {
    /// Emit an event to the event system
    async fn emit_event(&self, ctx: &ServiceContext, event: ServiceEvent) -> Result<()>;
}

/// Service event for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

/// Trait for services that can be cached
#[async_trait]
pub trait Cacheable<K, V> {
    /// Get cached value
    async fn get_cached(&self, key: &K) -> Result<Option<V>>;

    /// Set cached value with optional TTL
    async fn set_cached(&self, key: &K, value: &V, ttl_seconds: Option<u64>) -> Result<()>;

    /// Invalidate cached value
    async fn invalidate_cache(&self, key: &K) -> Result<()>;

    /// Clear all cached values for this service
    async fn clear_cache(&self) -> Result<()>;
}

/// Trait for services that support batch operations
#[async_trait]
pub trait BatchOperations<T, CreateInput, UpdateInput> {
    /// Create multiple entities in a transaction
    async fn batch_create(
        &self,
        ctx: &ServiceContext,
        inputs: Vec<CreateInput>,
    ) -> Result<ServiceResult<Vec<T>>>;

    /// Update multiple entities in a transaction
    async fn batch_update(
        &self,
        ctx: &ServiceContext,
        updates: Vec<UpdateInput>,
    ) -> Result<ServiceResult<Vec<T>>>;

    /// Delete multiple entities in a transaction
    async fn batch_delete(&self, ctx: &ServiceContext, ids: Vec<Uuid>)
    -> Result<ServiceResult<()>>;
}

/// Trait for services that support transaction management
#[async_trait]
pub trait Transactional {
    /// Execute operation within a database transaction
    async fn with_transaction<F, R>(&self, ctx: &ServiceContext, operation: F) -> Result<R>
    where
        F: FnOnce(&ServiceContext) -> Result<R> + Send + 'static,
        R: Send + 'static;
}

/// Trait for services that integrate with Kameo actors
// TODO: Fix the ActorIntegrated trait since it was entirely removed since Kameo's Actor trait requires Self: Sized and can't be used as dyn Actor.
// STRATEGY: Need to use concrete actor types instead of trait objects.
// #[async_trait]
// pub trait ActorIntegrated {
//     /// Get actor reference for this service's actor
//     async fn get_actor_ref(&self, ctx: &ServiceContext) -> Result<Option<ActorRef<dyn Actor>>>;

//     /// Send message to actor and wait for response
//     async fn ask_actor<M, R>(&self, ctx: &ServiceContext, message: M) -> Result<R>
//     where
//         M: Message<Reply = R> + Send + 'static,
//         R: Send + 'static;

/// Send message to actor without waiting for response
//     async fn tell_actor<M>(&self, ctx: &ServiceContext, message: M) -> Result<()>
//     where
//         M: Message + Send + 'static;
// }

/// Trait for services that support validation
#[async_trait]
pub trait Validatable<T> {
    /// Validate entity before create/update operations
    async fn validate(&self, ctx: &ServiceContext, entity: &T) -> Result<ValidationResult>;
}

/// Result of validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub code: String,
    pub message: String,
}

/// Trait for services that support search operations
#[async_trait]
pub trait Searchable<T, SearchQuery> {
    /// Perform full-text or semantic search
    async fn search(
        &self,
        ctx: &ServiceContext,
        query: SearchQuery,
    ) -> Result<ServiceResult<Vec<T>>>;
}

/// Base search query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub filters: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

/// Sort order for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Trait for services that support aggregation operations
#[async_trait]
pub trait Aggregatable<AggregateQuery, AggregateResult> {
    /// Perform aggregation operation
    async fn aggregate(
        &self,
        ctx: &ServiceContext,
        query: AggregateQuery,
    ) -> Result<ServiceResult<AggregateResult>>;
}

/// Helper trait for creating service contexts
pub trait ServiceContextBuilder {
    /// Create a new service context
    fn build(
        db: Pool<Sqlite>,
        actor_system: Arc<ActorSystem>,
        auth_context: Option<AuthContext>,
        workspace_id: Option<Uuid>,
    ) -> ServiceContext {
        ServiceContext {
            db,
            actor_system,
            auth_context,
            request_id: Uuid::new_v4(),
            workspace_id,
        }
    }
}

impl ServiceContextBuilder for ServiceContext {}

/// Helper macros for implementing common service patterns
#[macro_export]
macro_rules! impl_basic_service {
    ($service:ty, $entity:ty, $create_input:ty, $update_input:ty, $filter:ty) => {
        #[async_trait::async_trait]
        impl Service<$entity, $create_input, $update_input, $filter> for $service {
            async fn create(
                &self,
                ctx: &ServiceContext,
                input: $create_input,
            ) -> Result<ServiceResult<$entity>> {
                self.create_impl(ctx, input).await
            }

            async fn get(
                &self,
                ctx: &ServiceContext,
                id: Uuid,
            ) -> Result<ServiceResult<Option<$entity>>> {
                self.get_impl(ctx, id).await
            }

            async fn update(
                &self,
                ctx: &ServiceContext,
                entity: $update_input,
            ) -> Result<ServiceResult<$entity>> {
                self.update_impl(ctx, entity).await
            }

            async fn delete(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<()>> {
                self.delete_impl(ctx, id).await
            }

            async fn list(
                &self,
                ctx: &ServiceContext,
                filter: $filter,
            ) -> Result<ServiceResult<Vec<$entity>>> {
                self.list_impl(ctx, filter).await
            }

            async fn count(
                &self,
                ctx: &ServiceContext,
                filter: $filter,
            ) -> Result<ServiceResult<i64>> {
                self.count_impl(ctx, filter).await
            }
        }
    };
}

/// Actor system wrapper for services
pub struct ActorSystem {
    // This would contain the actual Kameo actor system
    // For now, we'll use a placeholder
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {}
    }
}
