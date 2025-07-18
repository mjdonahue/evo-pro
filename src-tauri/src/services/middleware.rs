use crate::error::Result;
use crate::logging;
use crate::services::traits::*;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Middleware trait for service operations
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process a service operation
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>>;
}

/// Next middleware in the chain
pub struct Next<'a, T> {
    pub(crate) middleware_index: usize,
    pub(crate) middlewares: &'a [Arc<dyn Middleware>],
    pub(crate) operation: Box<dyn FnOnce(&'a ServiceContext) -> Result<ServiceResult<T>> + Send + 'a>,
}

impl<'a, T: Send + 'static> Next<'a, T> {
    /// Create a new Next middleware
    pub fn new(
        middlewares: &'a [Arc<dyn Middleware>],
        operation: impl FnOnce(&'a ServiceContext) -> Result<ServiceResult<T>> + Send + 'a,
    ) -> Self {
        Self {
            middleware_index: 0,
            middlewares,
            operation: Box::new(operation),
        }
    }

    /// Run the next middleware in the chain
    pub async fn run(&self, ctx: &'a ServiceContext) -> Result<ServiceResult<T>> {
        if self.middleware_index < self.middlewares.len() {
            let middleware = &self.middlewares[self.middleware_index];
            let next = Self {
                middleware_index: self.middleware_index + 1,
                middlewares: self.middlewares,
                operation: self.operation.clone(),
            };
            middleware.process(ctx, next).await
        } else {
            (self.operation)(ctx)
        }
    }
}

/// Middleware stack for service operations
#[derive(Clone)]
pub struct MiddlewareStack {
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
}

impl MiddlewareStack {
    /// Create a new middleware stack
    pub fn new() -> Self {
        Self {
            middlewares: Arc::new(Vec::new()),
        }
    }

    /// Add a middleware to the stack
    pub fn with(mut self, middleware: impl Middleware + 'static) -> Self {
        let middlewares = Arc::make_mut(&mut self.middlewares);
        middlewares.push(Arc::new(middleware));
        self
    }

    /// Run the middleware stack
    pub async fn run<'a, T: Send + 'static>(
        &'a self,
        ctx: &'a ServiceContext,
        operation: impl FnOnce(&'a ServiceContext) -> Result<ServiceResult<T>> + Send + 'a,
    ) -> Result<ServiceResult<T>> {
        let next = Next::new(&self.middlewares, operation);
        next.run(ctx).await
    }
}

/// Logging middleware
pub struct LoggingMiddleware {
    pub log_level: crate::logging::LogLevel,
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>> {
        let operation_name = std::any::type_name::<T>();

        // Create a logger for this operation
        let logger = crate::logging::OperationLogger::from_service_context(operation_name, ctx)
            .with_log_start_end(true);

        // Start the operation
        logger.start();

        // Execute the operation
        let result = next.run(ctx).await;

        // Log the result
        match &result {
            Ok(service_result) => {
                // Log success with metrics if available
                if let Some(metadata) = &service_result.metadata {
                    logger.info(&format!(
                        "Operation completed successfully in {}ms with {} actor interactions",
                        metadata.execution_time_ms,
                        metadata.actor_interactions.len()
                    ));

                    // Log detailed actor interactions at debug level
                    if !metadata.actor_interactions.is_empty() {
                        for interaction in &metadata.actor_interactions {
                            logger.debug(&format!(
                                "Actor interaction: {} - {} ({}ms, success: {})",
                                interaction.actor_type,
                                interaction.operation,
                                interaction.duration_ms,
                                interaction.success
                            ));
                        }
                    }

                    // Log if this was a cache hit
                    if metadata.cache_hit {
                        logger.debug("Result was served from cache");
                    }
                } else {
                    logger.info("Operation completed successfully");
                }
            }
            Err(err) => {
                // Log error (the error will be logged in detail by the ErrorHandlingMiddleware)
                logger.error(&format!("Operation failed: {}", err));
            }
        }

        // Return the result (logger will log completion on drop)
        result
    }
}

/// Caching middleware
pub struct CachingMiddleware {
    pub cache_provider: Arc<dyn CacheProvider>,
    pub ttl_seconds: Option<u64>,
}

#[async_trait]
impl Middleware for CachingMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Send + 'static,
    {
        // Generate cache key based on operation and parameters
        let cache_key = format!("cache:{}:{}", std::any::type_name::<T>(), ctx.request_id);

        // Try to get from cache
        if let Some(cached_json) = self.cache_provider.get(&cache_key).await? {
            if let Ok(cached_data) = serde_json::from_str::<T>(&cached_json) {
                return Ok(ServiceResult {
                    data: cached_data,
                    metadata: Some(ServiceMetadata {
                        request_id: ctx.request_id,
                        execution_time_ms: 0,
                        cache_hit: true,
                        actor_interactions: vec![],
                    }),
                });
            }
        }

        // Execute the operation
        let result = next.run(ctx).await?;

        // Cache the result
        if let Ok(result_json) = serde_json::to_string(&result.data) {
            self.cache_provider
                .set(&cache_key, &result_json, self.ttl_seconds)
                .await?;
        }

        Ok(result)
    }
}

/// Metrics middleware
pub struct MetricsMiddleware;

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>> {
        let start = Instant::now();
        let operation_name = std::any::type_name::<T>();

        // Create a log context for metrics
        let log_context = logging::LogContext::from_service_context(ctx)
            .with_operation(operation_name)
            .with_context("metric_type", "service_operation");

        // Execute the operation
        let result = next.run(ctx).await;

        // Calculate duration
        let duration = start.elapsed();
        let duration_ms = duration.as_millis() as u64;
        let log_context = log_context.with_duration(duration);

        // Record metrics
        // In a real implementation, this would send metrics to a metrics system
        match &result {
            Ok(service_result) => {
                // Log basic metrics
                logging::debug_with_context(
                    &format!("Operation metrics: {}ms", duration_ms),
                    &log_context
                );

                // Log detailed metrics if available
                if let Some(metadata) = &service_result.metadata {
                    let mut metrics_context = log_context.clone()
                        .with_context("execution_time_ms", metadata.execution_time_ms.to_string())
                        .with_context("cache_hit", metadata.cache_hit.to_string())
                        .with_context("actor_interactions", metadata.actor_interactions.len().to_string());

                    logging::debug_with_context(
                        "Detailed operation metrics",
                        &metrics_context
                    );

                    // In a real implementation, we would send these metrics to a metrics system
                    // For now, we just log them
                }
            }
            Err(_) => {
                logging::debug_with_context(
                    &format!("Failed operation metrics: {}ms", duration_ms),
                    &log_context.with_context("status", "error")
                );
            }
        }

        result
    }
}

/// Error handling middleware
pub struct ErrorHandlingMiddleware;

#[async_trait]
impl Middleware for ErrorHandlingMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>> {
        // Execute the operation
        let result = next.run(ctx).await;

        // Handle errors
        match result {
            Ok(service_result) => Ok(service_result),
            Err(err) => {
                // Enrich the error with context from the service context
                let enriched_err = err.enrich(|context| {
                    let mut ctx_builder = context
                        .with_operation(std::any::type_name::<T>());

                    // Add request ID if available
                    if let Some(request_id) = ctx.request_id {
                        ctx_builder = ctx_builder.with_request_id(request_id);

                        // Try to extract correlation ID from request ID
                        // Format: "service_name.method_name:uuid"
                        if request_id.contains(":") {
                            let parts: Vec<&str> = request_id.split(":").collect();
                            if parts.len() > 1 {
                                ctx_builder = ctx_builder.with_correlation_id(parts[1]);
                            }
                        }
                    }

                    // Add workspace ID if available
                    if let Some(workspace_id) = ctx.workspace_id {
                        ctx_builder = ctx_builder.with_workspace_id(workspace_id);
                    }

                    // Add user ID if available
                    if let Some(auth_context) = &ctx.auth_context {
                        ctx_builder = ctx_builder.with_user_id(auth_context.participant_id.to_string());
                    }

                    ctx_builder
                });

                // Log the enriched error
                enriched_err.log();

                // In a real implementation, this could do more sophisticated error handling
                // such as retrying, transforming errors, etc.
                if enriched_err.is_retriable() {
                    // For retriable errors, we could implement retry logic here
                    // For now, just log that it's retriable
                    tracing::info!("Error is retriable, but retry logic is not implemented yet");
                }

                Err(enriched_err)
            }
        }
    }
}

/// Transaction middleware
pub struct TransactionMiddleware;

#[async_trait]
impl Middleware for TransactionMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>> {
        // Start a transaction
        let mut tx = ctx.db.begin().await?;

        // Create a new context with the transaction
        let tx_ctx = ServiceContext {
            db: tx.into(),
            actor_system: ctx.actor_system.clone(),
            auth_context: ctx.auth_context.clone(),
            request_id: ctx.request_id,
            workspace_id: ctx.workspace_id,
        };

        // Execute the operation
        let result = next.run(&tx_ctx).await;

        // Commit or rollback the transaction
        match &result {
            Ok(_) => {
                tx.commit().await?;
            }
            Err(_) => {
                tx.rollback().await?;
            }
        }

        result
    }
}

/// Authorization middleware
pub struct AuthorizationMiddleware {
    pub required_permissions: Vec<String>,
}

#[async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>> {
        // Check if the user has the required permissions
        if let Some(auth_context) = &ctx.auth_context {
            for permission in &self.required_permissions {
                if !auth_context.permissions.contains(&permission.to_string()) {
                    // Create a contextual error with detailed information
                    return Err(contextual_error!(
                        format!("Missing required permission: {}", permission),
                        .with_category(crate::error::ErrorCategory::Authorization)
                        .with_severity(crate::error::ErrorSeverity::Critical)
                        .with_operation(std::any::type_name::<T>())
                        .with_user_id(auth_context.participant_id.to_string())
                        .with_context("required_permission", permission)
                        .with_context("available_permissions", auth_context.permissions.join(", "))
                        .with_user_action("Contact an administrator to request the required permission")
                        .with_developer_action("Check permission requirements for this operation")
                    ));
                }
            }
        } else if !self.required_permissions.is_empty() {
            // Create a contextual error with detailed information
            return Err(contextual_error!(
                "No authentication context provided",
                .with_category(crate::error::ErrorCategory::Authentication)
                .with_severity(crate::error::ErrorSeverity::Critical)
                .with_operation(std::any::type_name::<T>())
                .with_context("required_permissions", self.required_permissions.join(", "))
                .with_user_action("Please log in to access this feature")
                .with_developer_action("Ensure authentication is performed before accessing protected resources")
            ));
        }

        // Execute the operation
        next.run(ctx).await
    }
}

/// Validation middleware
pub struct ValidationMiddleware<T> {
    pub validator: Box<dyn Fn(&T) -> Result<()> + Send + Sync>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ValidationMiddleware<T> {
    pub fn new(validator: impl Fn(&T) -> Result<()> + Send + Sync + 'static) -> Self {
        Self {
            validator: Box::new(validator),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Middleware for ValidationMiddleware<T> {
    async fn process<'a, R>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, R>,
    ) -> Result<ServiceResult<R>> {
        // Execute the operation
        next.run(ctx).await
    }
}

/// Create a default middleware stack with common middlewares
pub fn default_middleware_stack(
    cache_provider: Option<Arc<dyn CacheProvider>>,
) -> MiddlewareStack {
    let mut stack = MiddlewareStack::new()
        .with(LoggingMiddleware {
            log_level: logging::LogLevel::Info,
        })
        .with(crate::services::logging::ServiceLoggingMiddleware::default())
        .with(MetricsMiddleware)
        .with(ErrorHandlingMiddleware);

    if let Some(cache) = cache_provider {
        stack = stack.with(CachingMiddleware {
            cache_provider: cache,
            ttl_seconds: Some(300), // 5 minutes
        });
    }

    // Note: We don't add DeclarativeTransactionMiddleware here because it requires
    // a transaction manager, which is part of the BaseService. Instead, users should
    // call BaseService::with_declarative_transactions() to add it.

    stack
}
