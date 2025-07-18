use crate::entities::{
    Task, TaskFilter, TaskImportance, TaskPriority, TaskStats, TaskStatus, TaskUrgency,
};
use crate::error::{AppError, Result};
use crate::services::core::*;
use crate::services::traits::*;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

/// Input for creating a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub plan_id: Uuid,
    pub participant_id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub due_date: Option<chrono::DateTime<Utc>>,
    pub priority: TaskPriority,
    pub urgency: TaskUrgency,
    pub importance: TaskImportance,
    pub conversation_id: Option<Uuid>,
    pub memory_id: Option<Uuid>,
    pub memory_type: crate::entities::tasks::MemoryType,
    pub document_id: Option<Uuid>,
    pub file_id: Option<Uuid>,
    pub url: Option<String>,
    pub primary_assignee_id: Option<Uuid>,
    pub metadata: Option<String>,
}

/// Input for updating a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub due_date: Option<chrono::DateTime<Utc>>,
    pub priority: Option<TaskPriority>,
    pub urgency: Option<TaskUrgency>,
    pub importance: Option<TaskImportance>,
    pub status: Option<TaskStatus>,
    pub primary_assignee_id: Option<Uuid>,
    pub metadata: Option<String>,
}

/// Task service implementation
#[derive(Clone)]
pub struct TaskService {
    base: BaseService,
}

impl TaskService {
    pub fn new(base: BaseService) -> Self {
        // Enable declarative transactions for the task service
        let base = base.with_declarative_transactions();
        Self { base }
    }

    /// Get transaction attributes for a method
    fn get_transaction_attributes(&self, method_name: &str) -> TransactionAttributes {
        match method_name {
            "create_impl" => TransactionAttributes {
                isolation_level: Some(IsolationLevel::ReadCommitted),
                propagation: PropagationBehavior::Required,
                read_only: false,
                timeout: None,
                name: Some("create_task".to_string()),
            },
            "update_impl" => TransactionAttributes {
                isolation_level: Some(IsolationLevel::ReadCommitted),
                propagation: PropagationBehavior::Required,
                read_only: false,
                timeout: None,
                name: Some("update_task".to_string()),
            },
            "delete_impl" => TransactionAttributes {
                isolation_level: Some(IsolationLevel::ReadCommitted),
                propagation: PropagationBehavior::Required,
                read_only: false,
                timeout: None,
                name: Some("delete_task".to_string()),
            },
            "get_impl" | "list_impl" | "count_impl" => TransactionAttributes {
                isolation_level: Some(IsolationLevel::ReadCommitted),
                propagation: PropagationBehavior::Supports,
                read_only: true,
                timeout: None,
                name: Some(format!("{}_task", method_name)),
            },
            _ => TransactionAttributes::default(),
        }
    }

    /// Implementation methods for the service trait
    pub async fn create_impl(
        &self,
        ctx: &ServiceContext,
        input: CreateTaskInput,
    ) -> Result<ServiceResult<Task>> {
        // Use middleware pattern to handle cross-cutting concerns
        self.base.run(ctx, |ctx| async move {
            // Validate input
            self.validate_create_input(&input)?;

            // Create new task entity
            let now = Utc::now();
            let task = Task {
                id: Uuid::new_v4(),
                plan_id: input.plan_id,
                participant_id: input.participant_id,
                workspace_id: input.workspace_id,
                title: input.title,
                description: input.description,
                start_time: input.start_time,
                end_time: input.end_time,
                due_date: input.due_date,
                priority: input.priority,
                urgency: input.urgency,
                importance: input.importance,
                status: TaskStatus::Pending,
                metadata: input.metadata,
                conversation_id: input.conversation_id,
                memory_id: input.memory_id,
                memory_type: input.memory_type,
                document_id: input.document_id,
                file_id: input.file_id,
                url: input.url,
                primary_assignee_id: input.primary_assignee_id,
                created_by_id: ctx
                    .auth_context
                    .as_ref()
                    .map(|auth| auth.participant_id)
                    .unwrap_or_else(Uuid::new_v4),
                created_at: now,
                updated_at: now,
            };

            // Save to database
            Task::create(&ctx.db, &task).await?;

            // Emit creation event
            if let Some(emitter) = &self.base.event_emitter {
                let event = ServiceEvent {
                    event_type: "task_created".to_string(),
                    entity_type: "task".to_string(),
                    entity_id: task.id,
                    workspace_id: Some(task.workspace_id),
                    data: serde_json::to_value(&task)?,
                    metadata: None,
                };
                emitter.emit(event).await?;
            }

            // Create service result
            let mut metrics = ServiceMetrics::new();

            // Notify actor system
            self.notify_task_created(ctx, &task, &mut metrics).await?;

            Ok(metrics.finish(task))
        }.await)
    }

    pub async fn get_impl(
        &self,
        ctx: &ServiceContext,
        id: Uuid,
    ) -> Result<ServiceResult<Option<Task>>> {
        // Use middleware pattern to handle cross-cutting concerns
        self.base.run(ctx, |ctx| async move {
            // Get from database
            let task = Task::get_by_id(&ctx.db, &id).await?;

            // Create service result
            let metrics = ServiceMetrics::new();
            Ok(metrics.finish(task))
        }.await)
    }

    pub async fn update_impl(
        &self,
        ctx: &ServiceContext,
        input: UpdateTaskInput,
    ) -> Result<ServiceResult<Task>> {
        // Use middleware pattern to handle cross-cutting concerns
        self.base.run(ctx, |ctx| async move {
            // Get existing task
            let mut task = Task::get_by_id(&ctx.db, &input.id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFoundError(format!("Task with ID {} not found", input.id))
                })?;

            // Apply updates
            if let Some(title) = input.title {
                task.title = title;
            }
            if let Some(description) = input.description {
                task.description = Some(description);
            }
            if let Some(end_time) = input.end_time {
                task.end_time = Some(end_time);
            }
            if let Some(due_date) = input.due_date {
                task.due_date = Some(due_date);
            }
            if let Some(priority) = input.priority {
                task.priority = priority;
            }
            if let Some(urgency) = input.urgency {
                task.urgency = urgency;
            }
            if let Some(importance) = input.importance {
                task.importance = importance;
            }
            if let Some(status) = input.status {
                task.status = status;
            }
            if let Some(primary_assignee_id) = input.primary_assignee_id {
                task.primary_assignee_id = Some(primary_assignee_id);
            }
            if let Some(metadata) = input.metadata {
                task.metadata = Some(metadata);
            }

            task.updated_at = Utc::now();

            // Save to database
            Task::update(&ctx.db, &task).await?;

            // Emit update event
            if let Some(emitter) = &self.base.event_emitter {
                let event = ServiceEvent {
                    event_type: "task_updated".to_string(),
                    entity_type: "task".to_string(),
                    entity_id: task.id,
                    workspace_id: Some(task.workspace_id),
                    data: serde_json::to_value(&task)?,
                    metadata: None,
                };
                emitter.emit(event).await?;
            }

            // Create service result
            let mut metrics = ServiceMetrics::new();

            // Notify actor system
            self.notify_task_updated(ctx, &task, &mut metrics).await?;

            Ok(metrics.finish(task))
        }.await)
    }

    pub async fn delete_impl(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<()>> {
        // Use middleware pattern to handle cross-cutting concerns
        self.base.run(ctx, |ctx| async move {
            // Get task for event emission
            let task = Task::get_by_id(&ctx.db, &id).await?;

            // Delete from database
            Task::delete(&ctx.db, &id).await?;

            // Emit deletion event
            if let (Some(emitter), Some(task)) = (&self.base.event_emitter, task) {
                let event = ServiceEvent {
                    event_type: "task_deleted".to_string(),
                    entity_type: "task".to_string(),
                    entity_id: id,
                    workspace_id: Some(task.workspace_id),
                    data: serde_json::to_value(&task)?,
                    metadata: None,
                };
                emitter.emit(event).await?;
            }

            // Create service result
            let mut metrics = ServiceMetrics::new();

            // Notify actor system
            self.notify_task_deleted(ctx, id, &mut metrics).await?;

            Ok(metrics.finish(()))
        }.await)
    }

    pub async fn list_impl(
        &self,
        ctx: &ServiceContext,
        filter: TaskFilter,
    ) -> Result<ServiceResult<Vec<Task>>> {
        // Use middleware pattern to handle cross-cutting concerns
        self.base.run(ctx, |ctx| async move {
            // Get tasks from database
            let tasks = Task::list(&ctx.db, &filter).await?;

            // Create service result
            let metrics = ServiceMetrics::new();
            Ok(metrics.finish(tasks))
        }.await)
    }

    pub async fn count_impl(
        &self,
        ctx: &ServiceContext,
        filter: TaskFilter,
    ) -> Result<ServiceResult<i64>> {
        // Use middleware pattern to handle cross-cutting concerns
        self.base.run(ctx, |ctx| async move {
            // For now, we'll count by listing (in production, this should be optimized)
            let tasks = Task::list(&ctx.db, &filter).await?;
            let count = tasks.len() as i64;

            // Create service result
            let metrics = ServiceMetrics::new();
            Ok(metrics.finish(count))
        }.await)
    }

    /// Get task statistics
    pub async fn get_stats(
        &self,
        ctx: &ServiceContext,
        workspace_id: Option<Uuid>,
    ) -> Result<ServiceResult<TaskStats>> {
        let metrics = ServiceMetrics::new();

        let stats = if let Some(workspace_id) = workspace_id {
            Task::get_workspace_task_stats(&self.base.db, &workspace_id).await?
        } else {
            Task::get_task_stats(&self.base.db).await?
        };

        Ok(metrics.finish(stats))
    }

    /// Start a task (change status to InProgress)
    pub async fn start_task(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<()>> {
        let mut metrics = ServiceMetrics::new();

        Task::start_task(&self.base.db, &id).await?;

        // Notify actor system
        self.notify_task_status_changed(ctx, id, TaskStatus::InProgress, &mut metrics)
            .await?;

        Ok(metrics.finish(()))
    }

    /// Complete a task (change status to Completed)
    pub async fn complete_task(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<()>> {
        let mut metrics = ServiceMetrics::new();

        Task::complete_task(&self.base.db, &id).await?;

        // Notify actor system
        self.notify_task_status_changed(ctx, id, TaskStatus::Completed, &mut metrics)
            .await?;

        Ok(metrics.finish(()))
    }

    /// Get overdue tasks
    pub async fn get_overdue_tasks(
        &self,
        ctx: &ServiceContext,
    ) -> Result<ServiceResult<Vec<Task>>> {
        let metrics = ServiceMetrics::new();

        let tasks = Task::get_overdue_tasks(&self.base.db).await?;

        Ok(metrics.finish(tasks))
    }

    /// Get high priority tasks
    pub async fn get_high_priority_tasks(
        &self,
        ctx: &ServiceContext,
    ) -> Result<ServiceResult<Vec<Task>>> {
        let metrics = ServiceMetrics::new();

        let tasks = Task::get_high_priority_tasks(&self.base.db).await?;

        Ok(metrics.finish(tasks))
    }

    /// Actor notification methods
    async fn notify_task_created(
        &self,
        ctx: &ServiceContext,
        task: &Task,
        metrics: &mut ServiceMetrics,
    ) -> Result<()> {
        let start = Instant::now();

        // Here you would send messages to relevant actors
        // For example, notify the agent responsible for the task

        let duration = start.elapsed().as_millis() as u64;
        metrics.add_interaction(
            "TaskActor".to_string(),
            "task_created".to_string(),
            duration,
            true,
        );

        Ok(())
    }

    async fn notify_task_updated(
        &self,
        ctx: &ServiceContext,
        task: &Task,
        metrics: &mut ServiceMetrics,
    ) -> Result<()> {
        let start = Instant::now();

        // Notify relevant actors about the update

        let duration = start.elapsed().as_millis() as u64;
        metrics.add_interaction(
            "TaskActor".to_string(),
            "task_updated".to_string(),
            duration,
            true,
        );

        Ok(())
    }

    async fn notify_task_deleted(
        &self,
        ctx: &ServiceContext,
        id: Uuid,
        metrics: &mut ServiceMetrics,
    ) -> Result<()> {
        let start = Instant::now();

        // Notify relevant actors about the deletion

        let duration = start.elapsed().as_millis() as u64;
        metrics.add_interaction(
            "TaskActor".to_string(),
            "task_deleted".to_string(),
            duration,
            true,
        );

        Ok(())
    }

    async fn notify_task_status_changed(
        &self,
        ctx: &ServiceContext,
        id: Uuid,
        status: TaskStatus,
        metrics: &mut ServiceMetrics,
    ) -> Result<()> {
        let start = Instant::now();

        // Notify relevant actors about status change

        let duration = start.elapsed().as_millis() as u64;
        metrics.add_interaction(
            "TaskActor".to_string(),
            "status_changed".to_string(),
            duration,
            true,
        );

        Ok(())
    }

    /// Validation methods
    fn validate_create_input(&self, input: &CreateTaskInput) -> Result<()> {
        let mut validator = ValidationBuilder::new();

        if input.title.trim().is_empty() {
            validator.add_error("title", "required", "Task title is required");
        }

        if input.title.len() > 255 {
            validator.add_error(
                "title",
                "too_long",
                "Task title must be 255 characters or less",
            );
        }

        if let Some(due_date) = input.due_date {
            if due_date <= Utc::now() {
                validator.add_warning("due_date", "in_past", "Due date is in the past");
            }
        }

        let result = validator.build();
        if !result.valid {
            return Err(AppError::ValidationError(
                result
                    .errors
                    .iter()
                    .map(|e| format!("{}: {}", e.field, e.message))
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        Ok(())
    }
}

// Implement the service trait
crate::impl_basic_service!(
    TaskService,
    Task,
    CreateTaskInput,
    UpdateTaskInput,
    TaskFilter
);

// Implement additional traits
#[async_trait]
impl EventEmitter for TaskService {
    async fn emit_event(&self, ctx: &ServiceContext, event: ServiceEvent) -> Result<()> {
        if let Some(emitter) = &self.base.event_emitter {
            emitter.emit(event).await
        } else {
            Ok(())
        }
    }
}

// Implement DeclarativeTransactional trait
impl DeclarativeTransactional for TaskService {
    fn get_transaction_attributes(&self, method_name: &str) -> Option<TransactionAttributes> {
        Some(self.get_transaction_attributes(method_name))
    }
}

#[async_trait]
impl Validatable<CreateTaskInput> for TaskService {
    async fn validate(
        &self,
        ctx: &ServiceContext,
        entity: &CreateTaskInput,
    ) -> Result<ValidationResult> {
        let mut validator = ValidationBuilder::new();

        // Add validation logic here
        if entity.title.trim().is_empty() {
            validator.add_error("title", "required", "Task title is required");
        }

        Ok(validator.build())
    }
}

// Implement the service trait for the registry
#[async_trait]
impl TaskServiceTrait for TaskService {
    async fn create(
        &self,
        ctx: &ServiceContext,
        input: CreateTaskInput,
    ) -> Result<ServiceResult<Task>> {
        self.create_impl(ctx, input).await
    }

    async fn get(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<Option<Task>>> {
        self.get_impl(ctx, id).await
    }

    async fn list(
        &self,
        ctx: &ServiceContext,
        filter: TaskFilter,
    ) -> Result<ServiceResult<Vec<Task>>> {
        self.list_impl(ctx, filter).await
    }

    async fn update(
        &self,
        ctx: &ServiceContext,
        input: UpdateTaskInput,
    ) -> Result<ServiceResult<Task>> {
        self.update_impl(ctx, input).await
    }

    async fn delete(&self, ctx: &ServiceContext, id: Uuid) -> Result<ServiceResult<()>> {
        self.delete_impl(ctx, id).await
    }
}

// Example actor messages (would be implemented with proper Kameo actors)
// Note: These would need proper Actor implementations to work with Kameo

// Example of using the transactional! macro directly
impl TaskService {
    /// Example of using the transactional! macro with default settings
    #[transactional]
    pub async fn transfer_task(
        &self,
        ctx: &ServiceContext,
        task_id: Uuid,
        new_assignee_id: Uuid
    ) -> Result<ServiceResult<Task>> {
        // This entire method will run in a transaction with default settings

        // Get the task
        let mut task = Task::get_by_id(&ctx.db, &task_id)
            .await?
            .ok_or_else(|| AppError::NotFoundError(format!("Task with ID {} not found", task_id)))?;

        // Update the assignee
        task.primary_assignee_id = Some(new_assignee_id);
        task.updated_at = Utc::now();

        // Save to database
        Task::update(&ctx.db, &task).await?;

        // Create service result
        let metrics = ServiceMetrics::new();
        Ok(metrics.finish(task))
    }

    /// Example of using the transactional! macro with custom settings
    #[transactional(
        isolation = IsolationLevel::ReadCommitted,
        propagation = PropagationBehavior::Required,
        read_only = false,
        timeout = 30,
        name = "batch_update_tasks"
    )]
    pub async fn batch_update_tasks(
        &self,
        ctx: &ServiceContext,
        task_ids: Vec<Uuid>,
        status: TaskStatus
    ) -> Result<ServiceResult<Vec<Task>>> {
        // This entire method will run in a transaction with custom settings

        let mut updated_tasks = Vec::new();

        // Update each task
        for task_id in task_ids {
            // Get the task
            let mut task = Task::get_by_id(&ctx.db, &task_id)
                .await?
                .ok_or_else(|| AppError::NotFoundError(format!("Task with ID {} not found", task_id)))?;

            // Update the status
            task.status = status;
            task.updated_at = Utc::now();

            // Save to database
            Task::update(&ctx.db, &task).await?;

            updated_tasks.push(task);
        }

        // Create service result
        let metrics = ServiceMetrics::new();
        Ok(metrics.finish(updated_tasks))
    }

    /// Example of using nested transactions with savepoints
    #[transactional(
        isolation = IsolationLevel::ReadCommitted,
        propagation = PropagationBehavior::Nested,
        read_only = false
    )]
    pub async fn process_task_with_subtasks(
        &self,
        ctx: &ServiceContext,
        parent_task_id: Uuid,
        subtask_titles: Vec<String>
    ) -> Result<ServiceResult<(Task, Vec<Task>)>> {
        // This will create a savepoint if called within an existing transaction

        // Get the parent task
        let parent_task = Task::get_by_id(&ctx.db, &parent_task_id)
            .await?
            .ok_or_else(|| AppError::NotFoundError(format!("Task with ID {} not found", parent_task_id)))?;

        let mut subtasks = Vec::new();

        // Create subtasks
        for title in subtask_titles {
            let input = CreateTaskInput {
                plan_id: parent_task.plan_id,
                participant_id: parent_task.participant_id,
                workspace_id: parent_task.workspace_id,
                title,
                description: Some(format!("Subtask of {}", parent_task.title)),
                start_time: Utc::now(),
                end_time: parent_task.end_time,
                due_date: parent_task.due_date,
                priority: parent_task.priority,
                urgency: parent_task.urgency,
                importance: parent_task.importance,
                conversation_id: parent_task.conversation_id,
                memory_id: parent_task.memory_id,
                memory_type: parent_task.memory_type.clone(),
                document_id: parent_task.document_id,
                file_id: parent_task.file_id,
                url: parent_task.url.clone(),
                primary_assignee_id: parent_task.primary_assignee_id,
                metadata: None,
            };

            // Create the subtask (this will use the current transaction)
            let result = self.create_impl(ctx, input).await?;
            subtasks.push(result.data);
        }

        // Create service result
        let metrics = ServiceMetrics::new();
        Ok(metrics.finish((parent_task, subtasks)))
    }
}
