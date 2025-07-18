use crate::error::Result;
use crate::services::core::*;
use crate::services::traits::*;
use async_trait::async_trait;
use std::sync::Arc;

/// Service composition patterns for complex operations that span multiple services
/// This module provides patterns for composing services together to perform complex operations

/// CompositeService pattern for operations that span multiple services
/// This pattern allows combining multiple services to perform a complex operation
/// while maintaining transaction boundaries and error handling
pub struct CompositeService {
    pub registry: ServiceRegistry,
    pub base: BaseService,
}

impl CompositeService {
    pub fn new(registry: ServiceRegistry, base: BaseService) -> Self {
        Self { registry, base }
    }

    /// Execute a composite operation with proper transaction handling
    pub async fn execute<T: Send + 'static>(
        &self,
        ctx: &ServiceContext,
        operation: impl FnOnce(&ServiceContext, &ServiceRegistry) -> Result<ServiceResult<T>> + Send + 'static,
    ) -> Result<ServiceResult<T>> {
        // Use the base service's middleware stack to handle cross-cutting concerns
        self.base.run(ctx, |ctx| {
            operation(ctx, &self.registry)
        }).await
    }
}

/// Facade pattern for simplifying complex operations
/// This pattern provides a simplified interface for complex operations
/// that may involve multiple services and steps
#[async_trait]
pub trait ServiceFacade: Send + Sync {
    /// Get the service registry
    fn registry(&self) -> &ServiceRegistry;
    
    /// Get the base service
    fn base(&self) -> &BaseService;
    
    /// Execute an operation with the facade
    async fn execute<T: Send + 'static>(
        &self,
        ctx: &ServiceContext,
        operation: impl FnOnce(&ServiceContext, &ServiceRegistry) -> Result<ServiceResult<T>> + Send + 'static,
    ) -> Result<ServiceResult<T>> {
        // Use the base service's middleware stack to handle cross-cutting concerns
        self.base().run(ctx, |ctx| {
            operation(ctx, self.registry())
        }).await
    }
}

/// Command pattern for encapsulating operations
/// This pattern encapsulates an operation as an object, allowing for
/// parameterization, queueing, logging, and undoable operations
#[async_trait]
pub trait Command: Send + Sync {
    /// The type of the command result
    type Output: Send;
    
    /// Execute the command
    async fn execute(&self, ctx: &ServiceContext) -> Result<ServiceResult<Self::Output>>;
    
    /// Undo the command (if supported)
    async fn undo(&self, ctx: &ServiceContext) -> Result<()> {
        // Default implementation does nothing
        // Commands should override this if they support undo
        Err(crate::error::AppError::OperationNotSupported(
            "Undo not supported for this command".to_string(),
        ))
    }
}

/// CommandExecutor for executing commands
pub struct CommandExecutor {
    pub registry: ServiceRegistry,
    pub base: BaseService,
}

impl CommandExecutor {
    pub fn new(registry: ServiceRegistry, base: BaseService) -> Self {
        Self { registry, base }
    }
    
    /// Execute a command with proper middleware handling
    pub async fn execute<C: Command>(
        &self,
        ctx: &ServiceContext,
        command: C,
    ) -> Result<ServiceResult<C::Output>> {
        // Use the base service's middleware stack to handle cross-cutting concerns
        self.base.run(ctx, |ctx| {
            command.execute(ctx)
        }).await
    }
    
    /// Execute multiple commands in sequence
    pub async fn execute_sequence<C: Command>(
        &self,
        ctx: &ServiceContext,
        commands: Vec<C>,
    ) -> Result<Vec<ServiceResult<C::Output>>> {
        let mut results = Vec::new();
        
        for command in commands {
            let result = self.execute(ctx, command).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Execute multiple commands in a transaction
    #[transactional]
    pub async fn execute_transaction<C: Command>(
        &self,
        ctx: &ServiceContext,
        commands: Vec<C>,
    ) -> Result<Vec<ServiceResult<C::Output>>> {
        self.execute_sequence(ctx, commands).await
    }
}

/// Example implementation of a composite operation using the CompositeService pattern
pub struct TaskWithNotificationOperation {
    pub task_id: uuid::Uuid,
    pub notification_message: String,
}

impl TaskWithNotificationOperation {
    pub async fn execute(
        self,
        composite: &CompositeService,
        ctx: &ServiceContext,
    ) -> Result<ServiceResult<crate::entities::Task>> {
        composite.execute(ctx, |ctx, registry| async move {
            // Get the task
            let task_result = registry.task_service.get(ctx, self.task_id).await?;
            let task = task_result.data.ok_or_else(|| {
                crate::error::AppError::NotFoundError(format!("Task with ID {} not found", self.task_id))
            })?;
            
            // Create a notification (this would use a notification service in a real implementation)
            // For now, we'll just log it
            tracing::info!("Notification: {}", self.notification_message);
            
            // Return the task
            let metrics = ServiceMetrics::new();
            Ok(metrics.finish(task))
        }.await)
    }
}

/// Example implementation of a command
pub struct CreateTaskCommand {
    pub input: crate::services::task::CreateTaskInput,
}

#[async_trait]
impl Command for CreateTaskCommand {
    type Output = crate::entities::Task;
    
    async fn execute(&self, ctx: &ServiceContext) -> Result<ServiceResult<Self::Output>> {
        // In a real implementation, this would get the task service from the registry
        // For now, we'll just create a placeholder implementation
        Err(crate::error::AppError::OperationNotSupported(
            "This is just an example command".to_string(),
        ))
    }
    
    async fn undo(&self, ctx: &ServiceContext) -> Result<()> {
        // In a real implementation, this would delete the task
        // For now, we'll just create a placeholder implementation
        Err(crate::error::AppError::OperationNotSupported(
            "This is just an example command".to_string(),
        ))
    }
}

/// Example implementation of a service facade
pub struct TaskManagementFacade {
    registry: ServiceRegistry,
    base: BaseService,
}

impl TaskManagementFacade {
    pub fn new(registry: ServiceRegistry, base: BaseService) -> Self {
        Self { registry, base }
    }
    
    /// Create a task and notify relevant users
    pub async fn create_task_with_notification(
        &self,
        ctx: &ServiceContext,
        input: crate::services::task::CreateTaskInput,
        notification_message: String,
    ) -> Result<ServiceResult<crate::entities::Task>> {
        self.execute(ctx, |ctx, registry| async move {
            // Create the task
            let task_result = registry.task_service.create(ctx, input).await?;
            let task = task_result.data;
            
            // Create a notification (this would use a notification service in a real implementation)
            // For now, we'll just log it
            tracing::info!("Notification: {}", notification_message);
            
            // Return the task
            let metrics = ServiceMetrics::new();
            Ok(metrics.finish(task))
        }.await)
    }
    
    /// Assign multiple tasks to a user
    #[transactional]
    pub async fn assign_tasks(
        &self,
        ctx: &ServiceContext,
        task_ids: Vec<uuid::Uuid>,
        assignee_id: uuid::Uuid,
    ) -> Result<ServiceResult<Vec<crate::entities::Task>>> {
        self.execute(ctx, |ctx, registry| async move {
            let mut updated_tasks = Vec::new();
            
            for task_id in task_ids {
                // Get the task
                let task_result = registry.task_service.get(ctx, task_id).await?;
                let mut task = task_result.data.ok_or_else(|| {
                    crate::error::AppError::NotFoundError(format!("Task with ID {} not found", task_id))
                })?;
                
                // Update the assignee
                let update_input = crate::services::task::UpdateTaskInput {
                    id: task_id,
                    title: None,
                    description: None,
                    end_time: None,
                    due_date: None,
                    priority: None,
                    urgency: None,
                    importance: None,
                    status: None,
                    primary_assignee_id: Some(assignee_id),
                    metadata: None,
                };
                
                let updated_result = registry.task_service.update(ctx, update_input).await?;
                updated_tasks.push(updated_result.data);
            }
            
            // Return the updated tasks
            let metrics = ServiceMetrics::new();
            Ok(metrics.finish(updated_tasks))
        }.await)
    }
}

#[async_trait]
impl ServiceFacade for TaskManagementFacade {
    fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }
    
    fn base(&self) -> &BaseService {
        &self.base
    }
}