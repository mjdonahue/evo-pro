use crate::error::Result;
use crate::services::traits::*;
use crate::services::middleware::*;
use async_trait::async_trait;
use sqlx::{Pool, Sqlite, Transaction, TransactionBehavior, Acquire};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read uncommitted isolation level
    ReadUncommitted,
    /// Read committed isolation level
    ReadCommitted,
    /// Repeatable read isolation level
    RepeatableRead,
    /// Serializable isolation level
    Serializable,
}

impl IsolationLevel {
    /// Convert to SQLx isolation level
    pub fn to_sqlx(&self) -> sqlx::IsolationLevel {
        match self {
            IsolationLevel::ReadUncommitted => sqlx::IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted => sqlx::IsolationLevel::ReadCommitted,
            IsolationLevel::RepeatableRead => sqlx::IsolationLevel::RepeatableRead,
            IsolationLevel::Serializable => sqlx::IsolationLevel::Serializable,
        }
    }
}

/// Transaction propagation behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationBehavior {
    /// Always create a new transaction
    Required,
    /// Create a new transaction if none exists
    RequiresNew,
    /// Use existing transaction if available, otherwise non-transactional
    Supports,
    /// Use existing transaction if available, otherwise error
    Mandatory,
    /// Execute non-transactionally regardless of existing transaction
    NotSupported,
    /// Error if a transaction exists
    Never,
    /// Use existing transaction if available, create savepoint if transaction exists
    Nested,
}

/// Transaction attributes for declarative transaction management
#[derive(Debug, Clone)]
pub struct TransactionAttributes {
    /// Transaction isolation level
    pub isolation_level: Option<IsolationLevel>,
    /// Transaction propagation behavior
    pub propagation: PropagationBehavior,
    /// Whether the transaction is read-only
    pub read_only: bool,
    /// Timeout in seconds
    pub timeout: Option<u64>,
    /// Transaction name for logging
    pub name: Option<String>,
}

impl Default for TransactionAttributes {
    fn default() -> Self {
        Self {
            isolation_level: None,
            propagation: PropagationBehavior::Required,
            read_only: false,
            timeout: None,
            name: None,
        }
    }
}

/// Transaction manager for declarative transaction management
#[async_trait]
pub trait TransactionManager: Send + Sync {
    /// Execute an operation within a transaction according to the specified attributes
    async fn execute_in_transaction<F, R>(
        &self,
        ctx: &ServiceContext,
        attributes: &TransactionAttributes,
        operation: F,
    ) -> Result<R>
    where
        F: FnOnce(&ServiceContext) -> Pin<Box<dyn Future<Output = Result<R>> + Send>> + Send + 'static,
        R: Send + 'static;
}

/// Default transaction manager implementation
pub struct DefaultTransactionManager;

#[async_trait]
impl TransactionManager for DefaultTransactionManager {
    async fn execute_in_transaction<F, R>(
        &self,
        ctx: &ServiceContext,
        attributes: &TransactionAttributes,
        operation: F,
    ) -> Result<R>
    where
        F: FnOnce(&ServiceContext) -> Pin<Box<dyn Future<Output = Result<R>> + Send>> + Send + 'static,
        R: Send + 'static,
    {
        // Check if we're already in a transaction
        let is_transaction = ctx.db.begin().await.is_ok();

        match (is_transaction, attributes.propagation) {
            // Already in a transaction
            (true, PropagationBehavior::Required) |
            (true, PropagationBehavior::Supports) |
            (true, PropagationBehavior::Mandatory) => {
                // Use existing transaction
                operation(ctx).await
            },

            // Already in a transaction, but we need a new one
            (true, PropagationBehavior::RequiresNew) => {
                // Start a new transaction
                let mut tx_options = sqlx::TransactionOptions::new();

                // Set isolation level if specified
                if let Some(isolation) = attributes.isolation_level {
                    tx_options = tx_options.isolation_level(isolation.to_sqlx());
                }

                // Set read-only if specified
                if attributes.read_only {
                    tx_options = tx_options.read_only();
                }

                // Start transaction
                let mut tx = tx_options.begin(&ctx.db).await?;

                // Create new context with transaction
                let tx_ctx = ServiceContext {
                    db: tx.into(),
                    actor_system: ctx.actor_system.clone(),
                    auth_context: ctx.auth_context.clone(),
                    request_id: ctx.request_id,
                    workspace_id: ctx.workspace_id,
                };

                // Execute operation
                let result = operation(&tx_ctx).await;

                // Commit or rollback
                match result {
                    Ok(value) => {
                        tx.commit().await?;
                        Ok(value)
                    },
                    Err(err) => {
                        tx.rollback().await?;
                        Err(err)
                    }
                }
            },

            // Already in a transaction, but we don't want one
            (true, PropagationBehavior::NotSupported) => {
                // This is a simplification - in a real implementation, we would
                // suspend the current transaction, execute non-transactionally,
                // and then resume the transaction
                operation(ctx).await
            },

            // Already in a transaction, but we're not allowed to be in one
            (true, PropagationBehavior::Never) => {
                Err(crate::error::AppError::TransactionError(
                    "Transaction exists but propagation is set to NEVER".to_string()
                ))
            },

            // Already in a transaction, and we want a nested one
            (true, PropagationBehavior::Nested) => {
                // SQLite doesn't support true nested transactions, but we can use savepoints
                let savepoint_name = format!("sp_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

                // Execute savepoint query
                sqlx::query(&format!("SAVEPOINT {}", savepoint_name))
                    .execute(&ctx.db)
                    .await?;

                // Execute operation
                let result = operation(ctx).await;

                // Release or rollback savepoint
                match &result {
                    Ok(_) => {
                        sqlx::query(&format!("RELEASE SAVEPOINT {}", savepoint_name))
                            .execute(&ctx.db)
                            .await?;
                    },
                    Err(_) => {
                        sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", savepoint_name))
                            .execute(&ctx.db)
                            .await?;
                    }
                }

                result
            },

            // Not in a transaction, but we need one
            (false, PropagationBehavior::Required) |
            (false, PropagationBehavior::RequiresNew) => {
                // Start a new transaction
                let mut tx_options = sqlx::TransactionOptions::new();

                // Set isolation level if specified
                if let Some(isolation) = attributes.isolation_level {
                    tx_options = tx_options.isolation_level(isolation.to_sqlx());
                }

                // Set read-only if specified
                if attributes.read_only {
                    tx_options = tx_options.read_only();
                }

                // Start transaction
                let mut tx = tx_options.begin(&ctx.db).await?;

                // Create new context with transaction
                let tx_ctx = ServiceContext {
                    db: tx.into(),
                    actor_system: ctx.actor_system.clone(),
                    auth_context: ctx.auth_context.clone(),
                    request_id: ctx.request_id,
                    workspace_id: ctx.workspace_id,
                };

                // Execute operation
                let result = operation(&tx_ctx).await;

                // Commit or rollback
                match result {
                    Ok(value) => {
                        tx.commit().await?;
                        Ok(value)
                    },
                    Err(err) => {
                        tx.rollback().await?;
                        Err(err)
                    }
                }
            },

            // Not in a transaction, and we don't need one
            (false, PropagationBehavior::Supports) |
            (false, PropagationBehavior::NotSupported) => {
                // Execute non-transactionally
                operation(ctx).await
            },

            // Not in a transaction, but we require one
            (false, PropagationBehavior::Mandatory) => {
                Err(crate::error::AppError::TransactionError(
                    "No transaction exists but propagation is set to MANDATORY".to_string()
                ))
            },

            // Not in a transaction, and we don't want one
            (false, PropagationBehavior::Never) => {
                // Execute non-transactionally
                operation(ctx).await
            },

            // Not in a transaction, but we want a nested one
            (false, PropagationBehavior::Nested) => {
                // Treat as REQUIRED since we're not in a transaction
                // Start a new transaction
                let mut tx_options = sqlx::TransactionOptions::new();

                // Set isolation level if specified
                if let Some(isolation) = attributes.isolation_level {
                    tx_options = tx_options.isolation_level(isolation.to_sqlx());
                }

                // Set read-only if specified
                if attributes.read_only {
                    tx_options = tx_options.read_only();
                }

                // Start transaction
                let mut tx = tx_options.begin(&ctx.db).await?;

                // Create new context with transaction
                let tx_ctx = ServiceContext {
                    db: tx.into(),
                    actor_system: ctx.actor_system.clone(),
                    auth_context: ctx.auth_context.clone(),
                    request_id: ctx.request_id,
                    workspace_id: ctx.workspace_id,
                };

                // Execute operation
                let result = operation(&tx_ctx).await;

                // Commit or rollback
                match result {
                    Ok(value) => {
                        tx.commit().await?;
                        Ok(value)
                    },
                    Err(err) => {
                        tx.rollback().await?;
                        Err(err)
                    }
                }
            },
        }
    }
}

/// Declarative transaction middleware
pub struct DeclarativeTransactionMiddleware {
    pub transaction_manager: Arc<dyn TransactionManager>,
}

impl DeclarativeTransactionMiddleware {
    pub fn new(transaction_manager: Arc<dyn TransactionManager>) -> Self {
        Self {
            transaction_manager,
        }
    }
}

#[async_trait]
impl Middleware for DeclarativeTransactionMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>> {
        // Extract method name from the request context
        // First try to get it from the request_id which might contain the method name
        // Format: "service_name.method_name:uuid"
        let method_name = if let Some(request_id) = ctx.request_id {
            if request_id.contains(".") && request_id.contains(":") {
                let parts: Vec<&str> = request_id.split(":").collect();
                if !parts.is_empty() {
                    let service_method = parts[0];
                    if service_method.contains(".") {
                        service_method.split(".").last().unwrap_or("unknown").to_string()
                    } else {
                        service_method.to_string()
                    }
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            // Fallback to type name-based approach
            let type_name = std::any::type_name::<T>();
            if type_name.contains("::") {
                type_name.split("::").last().unwrap_or("unknown").to_string()
            } else {
                type_name.to_string()
            }
        };

        // Get attributes from the service if it implements DeclarativeTransactional
        // Otherwise, use default attributes
        let attributes = if let Some(service) = next.service.as_any().downcast_ref::<Box<dyn DeclarativeTransactional>>() {
            service.get_transaction_attributes(method_name).unwrap_or_default()
        } else {
            // For now, we'll use a simple heuristic based on the method name
            // Read operations are read-only, write operations are not
            let is_read = method_name.starts_with("get") || 
                          method_name.starts_with("list") || 
                          method_name.starts_with("count") ||
                          method_name.starts_with("find") ||
                          method_name.starts_with("search");

            TransactionAttributes {
                isolation_level: Some(IsolationLevel::ReadCommitted),
                propagation: if is_read { PropagationBehavior::Supports } else { PropagationBehavior::Required },
                read_only: is_read,
                timeout: None,
                name: Some(method_name.to_string()),
            }
        };

        // Execute the operation within a transaction
        self.transaction_manager.execute_in_transaction(
            ctx,
            &attributes,
            |ctx| Box::pin(async move { next.run(ctx).await }),
        ).await
    }
}

/// Trait for services that support declarative transaction management
#[async_trait]
pub trait DeclarativeTransactional {
    /// Get transaction attributes for a method
    fn get_transaction_attributes(&self, method_name: &str) -> Option<TransactionAttributes>;
}

/// Extension trait for BaseService to add declarative transaction support
pub trait TransactionalServiceExt {
    /// Add declarative transaction support to the service
    fn with_declarative_transactions(self, transaction_manager: Arc<dyn TransactionManager>) -> Self;
}

/// Macro to define transaction attributes for a method
/// 
/// This macro can be used in three ways:
/// 
/// 1. Simple form for synchronous methods:
///    ```
///    #[transactional]
///    pub fn my_method(&self, ctx: &ServiceContext, param: Type) -> Result<T> {
///        // Method body
///    }
///    ```
/// 
/// 2. Simple form for async methods:
///    ```
///    #[transactional]
///    pub async fn my_method(&self, ctx: &ServiceContext, param: Type) -> Result<T> {
///        // Method body
///    }
///    ```
/// 
/// 3. Detailed form with transaction attributes:
///    ```
///    #[transactional(
///        isolation = IsolationLevel::ReadCommitted,
///        propagation = PropagationBehavior::Required,
///        read_only = false,
///        timeout = 30,
///        name = "my_custom_transaction_name"
///    )]
///    pub async fn my_method(&self, ctx: &ServiceContext, param: Type) -> Result<T> {
///        // Method body
///    }
///    ```
#[macro_export]
macro_rules! transactional {
    // Simple form for synchronous methods
    (
        $(#[$attr:meta])*
        $vis:vis fn $name:ident $(<$($lt:lifetime),*>)? (
            &$self:ident, $ctx:ident : &$ctx_type:ty $(, $param:ident : $type:ty)*
        ) -> $ret:ty $body:block
    ) => {
        $(#[$attr])*
        $vis fn $name $(<$($lt),*>)? (
            &$self, $ctx: &$ctx_type $(, $param : $type)*
        ) -> $ret {
            // Default transaction attributes
            let attributes = crate::services::transaction::TransactionAttributes {
                isolation_level: None,
                propagation: crate::services::transaction::PropagationBehavior::Required,
                read_only: false,
                timeout: None,
                name: Some(stringify!($name).to_string()),
            };

            // Execute within transaction
            $self.base.transaction_manager.execute_in_transaction(
                $ctx,
                &attributes,
                |ctx| Box::pin(async move {
                    $body
                }),
            ).await
        }
    };

    // Simple form for async methods
    (
        $(#[$attr:meta])*
        $vis:vis async fn $name:ident $(<$($lt:lifetime),*>)? (
            &$self:ident, $ctx:ident : &$ctx_type:ty $(, $param:ident : $type:ty)*
        ) -> $ret:ty $body:block
    ) => {
        $(#[$attr])*
        $vis async fn $name $(<$($lt),*>)? (
            &$self, $ctx: &$ctx_type $(, $param : $type)*
        ) -> $ret {
            // Default transaction attributes
            let attributes = crate::services::transaction::TransactionAttributes {
                isolation_level: None,
                propagation: crate::services::transaction::PropagationBehavior::Required,
                read_only: false,
                timeout: None,
                name: Some(stringify!($name).to_string()),
            };

            // Execute within transaction
            $self.base.transaction_manager.execute_in_transaction(
                $ctx,
                &attributes,
                |ctx| Box::pin(async move {
                    $body
                }),
            ).await
        }
    };

    // Detailed form with transaction attributes
    (
        isolation = $isolation:expr,
        propagation = $propagation:expr,
        read_only = $read_only:expr
        $(, timeout = $timeout:expr)?
        $(, name = $name:expr)?

        $(#[$attr:meta])*
        $vis:vis async fn $fn_name:ident $(<$($lt:lifetime),*>)? (
            &$self:ident, $ctx:ident : &$ctx_type:ty $(, $param:ident : $type:ty)*
        ) -> $ret:ty $body:block
    ) => {
        $(#[$attr])*
        $vis async fn $fn_name $(<$($lt),*>)? (
            &$self, $ctx: &$ctx_type $(, $param : $type)*
        ) -> $ret {
            // Transaction attributes
            let attributes = crate::services::transaction::TransactionAttributes {
                isolation_level: Some($isolation),
                propagation: $propagation,
                read_only: $read_only,
                timeout: $(Some($timeout),)? $(None,)?
                name: $(Some($name.to_string()),)? $(Some(stringify!($fn_name).to_string()),)?
            };

            // Execute within transaction
            $self.base.transaction_manager.execute_in_transaction(
                $ctx,
                &attributes,
                |ctx| Box::pin(async move {
                    $body
                }),
            ).await
        }
    };

    // Attribute-style macro for async methods
    (
        $(
            $key:ident = $value:expr
        ),*
    ) => {
        compile_error!("The transactional attribute must be applied to a function");
    };

    // Attribute-style macro for async methods with no parameters
    (
        $(
            $key:ident = $value:expr
        ),*
    )
    => {
        compile_error!("The transactional attribute must be applied to a function");
    };
};
}
