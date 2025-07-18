use crate::error::{AppError, Result};
use crate::services::traits::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Transaction};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Base service implementation with common functionality
#[derive(Clone)]
pub struct BaseService {
    pub db: Pool<Sqlite>,
    pub actor_system: Arc<ActorSystem>,
    pub cache: Option<Arc<dyn CacheProvider>>,
    pub event_emitter: Option<Arc<dyn ServiceEventEmitter>>,
    pub middleware_stack: MiddlewareStack,
    pub transaction_manager: Arc<dyn TransactionManager>,
}

impl BaseService {
    pub fn new(
        db: Pool<Sqlite>,
        actor_system: Arc<ActorSystem>,
        cache: Option<Arc<dyn CacheProvider>>,
        event_emitter: Option<Arc<dyn ServiceEventEmitter>>,
    ) -> Self {
        // Create default middleware stack
        let middleware_stack = default_middleware_stack(cache.clone());

        // Create default transaction manager
        let transaction_manager = Arc::new(DefaultTransactionManager) as Arc<dyn TransactionManager>;

        Self {
            db,
            actor_system,
            cache,
            event_emitter,
            middleware_stack,
            transaction_manager,
        }
    }

    /// Create a service result with execution metadata
    pub fn create_result<T>(
        &self,
        data: T,
        start_time: Instant,
        interactions: Vec<ActorInteraction>,
    ) -> ServiceResult<T> {
        let execution_time = start_time.elapsed();
        ServiceResult {
            data,
            metadata: Some(ServiceMetadata {
                request_id: Uuid::new_v4(),
                execution_time_ms: execution_time.as_millis() as u64,
                cache_hit: false,
                actor_interactions: interactions,
            }),
        }
    }

    /// Create a service result with cache hit metadata
    pub fn create_cached_result<T>(&self, data: T) -> ServiceResult<T> {
        ServiceResult {
            data,
            metadata: Some(ServiceMetadata {
                request_id: Uuid::new_v4(),
                execution_time_ms: 0,
                cache_hit: true,
                actor_interactions: vec![],
            }),
        }
    }

    /// Run an operation through the middleware stack
    pub async fn run<'a, T: Send + 'static>(
        &'a self,
        ctx: &'a ServiceContext,
        operation: impl FnOnce(&'a ServiceContext) -> Result<ServiceResult<T>> + Send + 'a,
    ) -> Result<ServiceResult<T>> {
        self.middleware_stack.run(ctx, operation).await
    }

    /// Create a new service with additional middleware
    pub fn with_middleware(mut self, middleware: impl Middleware + 'static) -> Self {
        self.middleware_stack = self.middleware_stack.with(middleware);
        self
    }

    /// Create a new service with transaction middleware
    pub fn with_transaction(self) -> Self {
        self.with_middleware(TransactionMiddleware)
    }

    /// Create a new service with authorization middleware
    pub fn with_authorization(self, required_permissions: Vec<String>) -> Self {
        self.with_middleware(AuthorizationMiddleware {
            required_permissions,
        })
    }

    /// Create a new service with declarative transaction support
    pub fn with_declarative_transactions(self) -> Self {
        self.with_middleware(DeclarativeTransactionMiddleware::new(self.transaction_manager.clone()))
    }

    /// Set a custom transaction manager
    pub fn with_transaction_manager(mut self, transaction_manager: Arc<dyn TransactionManager>) -> Self {
        self.transaction_manager = transaction_manager;
        self
    }
}

/// Implementation of TransactionalServiceExt for BaseService
impl TransactionalServiceExt for BaseService {
    fn with_declarative_transactions(self, transaction_manager: Arc<dyn TransactionManager>) -> Self {
        self.with_transaction_manager(transaction_manager)
            .with_middleware(DeclarativeTransactionMiddleware::new(transaction_manager))
    }
}

/// Transaction wrapper for service operations
pub struct ServiceTransaction<'a> {
    pub tx: Transaction<'a, Sqlite>,
    pub ctx: ServiceContext,
}

impl<'a> ServiceTransaction<'a> {
    pub async fn commit(self) -> Result<()> {
        Ok(self.tx.commit().await?)
    }

    pub async fn rollback(self) -> Result<()> {
        Ok(self.tx.rollback().await?)
    }
}

/// Cache provider trait for service caching
#[async_trait]
pub trait CacheProvider: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self, pattern: &str) -> Result<()>;
}

/// Event emitter trait for service events
#[async_trait]
pub trait ServiceEventEmitter: Send + Sync {
    async fn emit(&self, event: ServiceEvent) -> Result<()>;
}

/// In-memory cache implementation for development
#[derive(Debug)]
pub struct InMemoryCache {
    // In a real implementation, this would use a proper cache like Redis
    // For now, we'll use a simple placeholder
}

#[async_trait]
impl CacheProvider for InMemoryCache {
    async fn get(&self, _key: &str) -> Result<Option<String>> {
        // Placeholder implementation
        Ok(None)
    }

    async fn set(&self, _key: &str, _value: &str, _ttl_seconds: Option<u64>) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }

    async fn clear(&self, _pattern: &str) -> Result<()> {
        Ok(())
    }
}

/// Event bus implementation using Kameo actors
#[derive(Clone)]
pub struct ActorEventEmitter {
    // Actor reference to event bus
    // event_bus: ActorRef<EventBusActor>,
}

#[async_trait]
impl ServiceEventEmitter for ActorEventEmitter {
    async fn emit(&self, _event: ServiceEvent) -> Result<()> {
        // Placeholder implementation
        // In a real implementation, this would send the event to the event bus actor
        Ok(())
    }
}

/// Service registry for managing all services
#[derive(Clone)]
pub struct ServiceRegistry {
    pub conversation_service: Arc<crate::services::conversation::ConversationService>,
    pub message_service: Arc<crate::services::message::MessageService>,
    pub task_service: Arc<crate::services::task::TaskService>,
    pub plan_service: Arc<crate::services::plan::PlanService>,
    pub agent_service: Arc<crate::services::agent::AgentService>,
    pub event_service: Arc<crate::services::events::EventService>,
}

impl ServiceRegistry {
    pub fn new(
        db: Pool<Sqlite>,
        actor_system: Arc<ActorSystem>,
        cache: Option<Arc<dyn CacheProvider>>,
        event_emitter: Option<Arc<dyn ServiceEventEmitter>>,
    ) -> Self {
        let base_service = BaseService::new(db.clone(), actor_system, cache, event_emitter);

        Self {
            conversation_service: Arc::new(
                crate::services::conversation::ConversationService::new(base_service.clone()),
            ),
            message_service: Arc::new(crate::services::message::MessageService::new(
                base_service.clone(),
            )),
            task_service: Arc::new(crate::services::task::TaskService::new(
                base_service.clone(),
            )),
            plan_service: Arc::new(crate::services::plan::PlanService::new(
                base_service.clone(),
            )),
            agent_service: Arc::new(crate::services::agent::AgentService::new(
                base_service.clone(),
            )),
            event_service: Arc::new(crate::services::events::EventService::new(base_service)),
        }
    }
}

/// Service factory for creating service instances
pub struct ServiceFactory;

impl ServiceFactory {
    pub fn create_registry(
        db: Pool<Sqlite>,
        enable_cache: bool,
        enable_events: bool,
    ) -> ServiceRegistry {
        let actor_system = Arc::new(ActorSystem::new());

        let cache = if enable_cache {
            Some(Arc::new(InMemoryCache {}) as Arc<dyn CacheProvider>)
        } else {
            None
        };

        let event_emitter = if enable_events {
            Some(Arc::new(ActorEventEmitter {}) as Arc<dyn ServiceEventEmitter>)
        } else {
            None
        };

        ServiceRegistry::new(db, actor_system, cache, event_emitter)
    }
}

/// Performance monitoring utilities
pub struct ServiceMetrics {
    pub start_time: Instant,
    pub interactions: Vec<ActorInteraction>,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            interactions: Vec::new(),
        }
    }

    pub fn add_interaction(
        &mut self,
        actor_type: String,
        operation: String,
        duration_ms: u64,
        success: bool,
    ) {
        self.interactions.push(ActorInteraction {
            actor_type,
            operation,
            duration_ms,
            success,
        });
    }

    pub fn finish<T>(self, data: T) -> ServiceResult<T> {
        let execution_time = self.start_time.elapsed();
        ServiceResult {
            data,
            metadata: Some(ServiceMetadata {
                request_id: Uuid::new_v4(),
                execution_time_ms: execution_time.as_millis() as u64,
                cache_hit: false,
                actor_interactions: self.interactions,
            }),
        }
    }
}

/// Pagination utilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl PaginationParams {
    pub fn new(limit: Option<u32>, offset: Option<u32>) -> Self {
        Self { limit, offset }
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20).min(100) // Default 20, max 100
    }

    pub fn offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }
}

/// Error handling utilities
pub trait ServiceErrorExt {
    fn into_service_error(self) -> AppError;
}

impl ServiceErrorExt for sqlx::Error {
    fn into_service_error(self) -> AppError {
        AppError::DatabaseError(self.to_string())
    }
}

/// Validation utilities
pub struct ValidationBuilder {
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
}

impl Default for ValidationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationBuilder {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, field: &str, code: &str, message: &str) -> &mut Self {
        self.errors.push(ValidationError {
            field: field.to_string(),
            code: code.to_string(),
            message: message.to_string(),
        });
        self
    }

    pub fn add_warning(&mut self, field: &str, code: &str, message: &str) -> &mut Self {
        self.warnings.push(ValidationWarning {
            field: field.to_string(),
            code: code.to_string(),
            message: message.to_string(),
        });
        self
    }

    pub fn build(self) -> ValidationResult {
        ValidationResult {
            valid: self.errors.is_empty(),
            errors: self.errors,
            warnings: self.warnings,
        }
    }
}

/// Service traits for type safety
pub trait ConversationServiceTrait: Send + Sync {
    // Methods will be defined in the conversation service
}

pub trait MessageServiceTrait: Send + Sync {
    // Methods will be defined in the message service
}

#[async_trait]
pub trait TaskServiceTrait: Send + Sync {
    // CRUD operations
    async fn create(
        &self,
        ctx: &ServiceContext,
        input: crate::services::task::CreateTaskInput,
    ) -> Result<ServiceResult<crate::entities::Task>>;
    async fn get(
        &self,
        ctx: &ServiceContext,
        id: Uuid,
    ) -> Result<ServiceResult<Option<crate::entities::Task>>>;
    async fn list(
        &self,
        ctx: &ServiceContext,
        filter: crate::entities::TaskFilter,
    ) -> Result<ServiceResult<Vec<crate::entities::Task>>>;
    async fn update(
        &self,
        ctx: &ServiceContext,
        input: crate::services::task::UpdateTaskInput,
    ) -> Result<ServiceResult<crate::entities::Task>>;
    async fn delete(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<()>>;
}

pub trait PlanServiceTrait: Send + Sync {
    // Methods will be defined in the plan service
}

pub trait AgentServiceTrait: Send + Sync {
    // Methods will be defined in the agent service
}

pub trait EventServiceTrait: Send + Sync {
    // Methods will be defined in the event service
}
