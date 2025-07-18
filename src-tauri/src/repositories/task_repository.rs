//! Task repository implementation
//!
//! This module provides a repository implementation for tasks that encapsulates
//! database access logic and uses the specialized query builder.

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Pool, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::entities::tasks::{Task, TaskFilter, TaskStatus};
use crate::error::{AppError, Result};
use crate::repositories::base::{BaseRepository, Repository};
use crate::repositories::query_builder::TaskQueryBuilder;

/// Task with related entities
#[derive(Debug, Clone)]
pub struct TaskWithRelations {
    /// The task
    pub task: Task,
    /// Workspace name (if available)
    pub workspace_name: Option<String>,
    /// Plan name (if available)
    pub plan_name: Option<String>,
    /// Assignee name (if available)
    pub assignee_name: Option<String>,
    /// Creator name (if available)
    pub creator_name: Option<String>,
}

/// Task repository implementation
pub struct TaskRepository {
    /// Base repository
    base: BaseRepository,
}

impl TaskRepository {
    /// Create a new task repository
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            base: BaseRepository::new(pool),
        }
    }

    /// Get the database connection pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.base.pool
    }

    /// Get tasks with related entities
    #[instrument(skip(self))]
    pub async fn get_tasks_with_relations(&self, filter: &TaskFilter) -> Result<Vec<TaskWithRelations>> {
        debug!("Getting tasks with relations: {:?}", filter);

        // Build a query that joins tasks with related tables
        let mut query = r#"
            SELECT 
                t.id as "task_id: _",
                t.title as "task_title",
                t.description as "task_description",
                t.status as "task_status: TaskStatus",
                t.start_time as "task_start_time: _",
                t.end_time as "task_end_time: _",
                t.due_date as "task_due_date: _",
                t.priority as "task_priority: TaskPriority",
                t.importance as "task_importance: TaskImportance",
                t.tags as "task_tags: _",
                t.url as "task_url",
                t.metadata as "task_metadata: _",
                t.created_at as "task_created_at: _",
                t.updated_at as "task_updated_at: _",
                t.created_by_id as "task_created_by_id: _",
                t.assignee_participant_id as "task_assignee_participant_id: _",
                t.workspace_id as "task_workspace_id: _",
                t.conversation_id as "task_conversation_id: _",
                t.memory_id as "task_memory_id: _",
                t.plan_id as "task_plan_id: _",
                t.document_id as "task_document_id: _",
                t.file_id as "task_file_id: _",
                w.name as "workspace_name",
                p.name as "plan_name",
                a.display_name as "assignee_name",
                c.display_name as "creator_name"
            FROM tasks t
            LEFT JOIN workspaces w ON t.workspace_id = w.id
            LEFT JOIN plans p ON t.plan_id = p.id
            LEFT JOIN participants a ON t.assignee_participant_id = a.id
            LEFT JOIN users c ON t.created_by_id = c.id
        "#.to_string();

        // Add WHERE clauses based on filter
        let mut conditions = Vec::new();

        if let Some(workspace_id) = filter.workspace_id {
            conditions.push(format!("t.workspace_id = '{}'", workspace_id));
        }

        if let Some(plan_id) = filter.plan_id {
            conditions.push(format!("t.plan_id = '{}'", plan_id));
        }

        if let Some(status) = filter.status {
            conditions.push(format!("t.status = {}", status as i32));
        }

        if let Some(true) = filter.active_only {
            conditions.push("t.status NOT IN (2, 3)".to_string()); // Not Completed or Failed
        }

        if let Some(true) = filter.overdue_only {
            conditions.push("t.due_date < datetime('now') AND t.status NOT IN (2, 3)".to_string());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Add ORDER BY
        query.push_str(" ORDER BY t.due_date ASC, t.priority DESC, t.importance DESC, t.title ASC");

        // Add LIMIT and OFFSET
        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));

            if let Some(offset) = filter.offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
        }

        // Execute the query
        let rows = sqlx::query(&query)
            .fetch_all(&self.base.pool)
            .await?;

        // Map the rows to TaskWithRelations objects
        let mut tasks = Vec::new();
        for row in rows {
            let task = Task {
                id: row.get("task_id"),
                title: row.get("task_title"),
                description: row.get("task_description"),
                status: row.get("task_status"),
                start_time: row.get("task_start_time"),
                end_time: row.get("task_end_time"),
                due_date: row.get("task_due_date"),
                priority: row.get("task_priority"),
                importance: row.get("task_importance"),
                tags: row.get("task_tags"),
                url: row.get("task_url"),
                metadata: row.get("task_metadata"),
                created_at: row.get("task_created_at"),
                updated_at: row.get("task_updated_at"),
                created_by_id: row.get("task_created_by_id"),
                assignee_participant_id: row.get("task_assignee_participant_id"),
                workspace_id: row.get("task_workspace_id"),
                conversation_id: row.get("task_conversation_id"),
                memory_id: row.get("task_memory_id"),
                plan_id: row.get("task_plan_id"),
                document_id: row.get("task_document_id"),
                file_id: row.get("task_file_id"),
            };

            let task_with_relations = TaskWithRelations {
                task,
                workspace_name: row.get("workspace_name"),
                plan_name: row.get("plan_name"),
                assignee_name: row.get("assignee_name"),
                creator_name: row.get("creator_name"),
            };

            tasks.push(task_with_relations);
        }

        Ok(tasks)
    }

    /// Update task status
    #[instrument(skip(self))]
    pub async fn update_status(&self, id: &Uuid, status: TaskStatus) -> Result<()> {
        debug!("Updating task status: id={}, status={:?}", id, status);
        let affected = sqlx::query!("UPDATE tasks SET status = ? WHERE id = ?", status, id)
            .execute(&self.base.pool)
            .await?;
        if affected.rows_affected() == 0 {
            return Err(AppError::not_found("Task", id));
        }
        Ok(())
    }

    /// Start task (mark as in progress)
    pub async fn start_task(&self, id: &Uuid) -> Result<()> {
        self.update_status(id, TaskStatus::InProgress).await
    }

    /// Complete task
    pub async fn complete_task(&self, id: &Uuid) -> Result<()> {
        self.update_status(id, TaskStatus::Completed).await
    }

    /// Fail task
    pub async fn fail_task(&self, id: &Uuid) -> Result<()> {
        self.update_status(id, TaskStatus::Failed).await
    }

    /// Get tasks by plan
    #[instrument(skip(self))]
    pub async fn get_tasks_by_plan(&self, plan_id: &Uuid) -> Result<Vec<Task>> {
        let filter = TaskFilter {
            plan_id: Some(*plan_id),
            ..Default::default()
        };

        self.list(&filter).await
    }

    /// Get overdue tasks
    #[instrument(skip(self))]
    pub async fn get_overdue_tasks(&self) -> Result<Vec<Task>> {
        let filter = TaskFilter {
            overdue_only: Some(true),
            ..Default::default()
        };

        self.list(&filter).await
    }

    /// Get high priority tasks
    #[instrument(skip(self))]
    pub async fn get_high_priority_tasks(&self) -> Result<Vec<Task>> {
        let filter = TaskFilter {
            priority: Some(crate::entities::tasks::TaskPriority::High),
            active_only: Some(true),
            ..Default::default()
        };

        self.list(&filter).await
    }

    /// Get task statistics
    #[instrument(skip(self))]
    pub async fn get_task_stats(&self) -> Result<TaskStats> {
        // Using the enhanced query builder with aggregations
        let mut builder = crate::repositories::query_builder::EnhancedQueryBuilder::new(
            "SELECT COUNT(*) as total_tasks"
        );

        // Add aggregations for different task categories
        builder.builder_mut().push(",
            COUNT(CASE WHEN status = 0 THEN 1 END) as pending_tasks,
            COUNT(CASE WHEN status = 1 THEN 1 END) as in_progress_tasks,
            COUNT(CASE WHEN status = 2 THEN 1 END) as completed_tasks,
            COUNT(CASE WHEN status = 3 THEN 1 END) as failed_tasks,
            COUNT(CASE WHEN priority = 2 THEN 1 END) as high_priority_tasks,
            COUNT(CASE WHEN importance = 2 THEN 1 END) as important_tasks,
            COUNT(CASE WHEN due_date < datetime('now') AND status NOT IN (2, 3) THEN 1 END) as overdue_tasks
        ");

        // Add FROM clause
        builder.builder_mut().push(" FROM tasks");

        // Execute the query
        let stats = sqlx::query_as!(
            TaskStatsRow,
            &builder.builder().sql(),
        )
        .fetch_one(&self.base.pool)
        .await?;

        Ok(TaskStats {
            pending_tasks: stats.pending_tasks.unwrap_or(0) as usize,
            in_progress_tasks: stats.in_progress_tasks.unwrap_or(0) as usize,
            completed_tasks: stats.completed_tasks.unwrap_or(0) as usize,
            failed_tasks: stats.failed_tasks.unwrap_or(0) as usize,
            high_priority_tasks: stats.high_priority_tasks.unwrap_or(0) as usize,
            important_tasks: stats.important_tasks.unwrap_or(0) as usize,
            overdue_tasks: stats.overdue_tasks.unwrap_or(0) as usize,
            total_tasks: stats.total_tasks.unwrap_or(0) as usize,
        })
    }

    /// Get detailed task statistics with aggregations by workspace
    #[instrument(skip(self))]
    pub async fn get_detailed_task_stats(&self) -> Result<Vec<WorkspaceTaskStats>> {
        // Using the enhanced query builder with aggregations and joins
        let mut builder = crate::repositories::query_builder::EnhancedQueryBuilder::new(
            "SELECT w.id as workspace_id, w.name as workspace_name"
        );

        // Add aggregations
        builder.builder_mut().push(",
            COUNT(t.id) as total_tasks,
            COUNT(CASE WHEN t.status = 0 THEN 1 END) as pending_tasks,
            COUNT(CASE WHEN t.status = 1 THEN 1 END) as in_progress_tasks,
            COUNT(CASE WHEN t.status = 2 THEN 1 END) as completed_tasks,
            COUNT(CASE WHEN t.status = 3 THEN 1 END) as failed_tasks,
            COUNT(CASE WHEN t.priority = 2 THEN 1 END) as high_priority_tasks,
            COUNT(CASE WHEN t.due_date < datetime('now') AND t.status NOT IN (2, 3) THEN 1 END) as overdue_tasks,
            AVG(CASE WHEN t.status = 2 THEN (julianday(t.end_time) - julianday(t.start_time)) * 24 * 60 END) as avg_completion_time_minutes
        ");

        // Add FROM clause with JOIN
        builder.builder_mut().push(" FROM workspaces w LEFT JOIN tasks t ON w.id = t.workspace_id");

        // Add GROUP BY
        builder.add_group_by(&["w.id", "w.name"]);

        // Add ORDER BY
        builder.add_order_by("w.name", crate::repositories::query_builder::OrderDirection::Asc);

        // Execute the query
        let rows = sqlx::query_as!(
            WorkspaceTaskStatsRow,
            &builder.builder().sql(),
        )
        .fetch_all(&self.base.pool)
        .await?;

        // Convert rows to WorkspaceTaskStats
        let stats = rows.into_iter().map(|row| {
            WorkspaceTaskStats {
                workspace_id: row.workspace_id,
                workspace_name: row.workspace_name,
                total_tasks: row.total_tasks.unwrap_or(0) as usize,
                pending_tasks: row.pending_tasks.unwrap_or(0) as usize,
                in_progress_tasks: row.in_progress_tasks.unwrap_or(0) as usize,
                completed_tasks: row.completed_tasks.unwrap_or(0) as usize,
                failed_tasks: row.failed_tasks.unwrap_or(0) as usize,
                high_priority_tasks: row.high_priority_tasks.unwrap_or(0) as usize,
                overdue_tasks: row.overdue_tasks.unwrap_or(0) as usize,
                avg_completion_time_minutes: row.avg_completion_time_minutes,
            }
        }).collect();

        Ok(stats)
    }

    /// Calculate task score (for prioritization)
    pub fn calculate_task_score(&self, task: &Task) -> u32 {
        let priority_weight = 3;
        let importance_weight = 4;

        let base_score = (task.priority as u32 * priority_weight)
            + (task.importance as u32 * importance_weight);

        // Add overdue penalty
        if let Some(due_date) = task.due_date {
            if due_date < Utc::now()
                && !matches!(task.status, TaskStatus::Completed | TaskStatus::Failed)
            {
                return base_score + 10; // High penalty for overdue
            }
        }

        base_score
    }
}

/// Task statistics row for query results
#[derive(Debug, sqlx::FromRow)]
struct TaskStatsRow {
    pending_tasks: Option<i64>,
    in_progress_tasks: Option<i64>,
    completed_tasks: Option<i64>,
    failed_tasks: Option<i64>,
    high_priority_tasks: Option<i64>,
    important_tasks: Option<i64>,
    overdue_tasks: Option<i64>,
    total_tasks: Option<i64>,
}

/// Workspace task statistics row for query results
#[derive(Debug, sqlx::FromRow)]
struct WorkspaceTaskStatsRow {
    workspace_id: Option<Uuid>,
    workspace_name: Option<String>,
    total_tasks: Option<i64>,
    pending_tasks: Option<i64>,
    in_progress_tasks: Option<i64>,
    completed_tasks: Option<i64>,
    failed_tasks: Option<i64>,
    high_priority_tasks: Option<i64>,
    overdue_tasks: Option<i64>,
    avg_completion_time_minutes: Option<f64>,
}

/// Task statistics
#[derive(Debug, Clone)]
pub struct TaskStats {
    /// Number of pending tasks
    pub pending_tasks: usize,
    /// Number of in-progress tasks
    pub in_progress_tasks: usize,
    /// Number of completed tasks
    pub completed_tasks: usize,
    /// Number of failed tasks
    pub failed_tasks: usize,
    /// Number of high priority tasks
    pub high_priority_tasks: usize,
    /// Number of important tasks
    pub important_tasks: usize,
    /// Number of overdue tasks
    pub overdue_tasks: usize,
    /// Total number of tasks
    pub total_tasks: usize,
}

/// Workspace task statistics
#[derive(Debug, Clone)]
pub struct WorkspaceTaskStats {
    /// Workspace ID
    pub workspace_id: Option<Uuid>,
    /// Workspace name
    pub workspace_name: Option<String>,
    /// Total number of tasks
    pub total_tasks: usize,
    /// Number of pending tasks
    pub pending_tasks: usize,
    /// Number of in-progress tasks
    pub in_progress_tasks: usize,
    /// Number of completed tasks
    pub completed_tasks: usize,
    /// Number of failed tasks
    pub failed_tasks: usize,
    /// Number of high priority tasks
    pub high_priority_tasks: usize,
    /// Number of overdue tasks
    pub overdue_tasks: usize,
    /// Average completion time in minutes
    pub avg_completion_time_minutes: Option<f64>,
}

#[async_trait]
impl Repository<Task, TaskFilter> for TaskRepository {
    /// Get multiple tasks by IDs in a single query
    #[instrument(skip(self))]
    async fn get_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Task>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Getting tasks by IDs: {:?}", ids);

        // Build a query with IN clause
        let mut query = r#"SELECT 
            id as "id: _", title, description, status as "status: TaskStatus", start_time as "start_time: _",
            end_time as "end_time: _", due_date as "due_date: _", priority as "priority: TaskPriority",
            importance as "importance: TaskImportance", tags as "tags: _", url, metadata as "metadata: _",
            created_at as "created_at: _", updated_at as "updated_at: _", created_by_id as "created_by_id: _",
            assignee_participant_id as "assignee_participant_id: _", workspace_id as "workspace_id: _",
            conversation_id as "conversation_id: _", memory_id as "memory_id: _", plan_id as "plan_id: _",
            document_id as "document_id: _", file_id as "file_id: _"
        FROM tasks WHERE id IN ("#.to_string();

        // Add placeholders for each ID
        let placeholders: Vec<String> = (0..ids.len()).map(|i| format!("${}", i + 1)).collect();
        query.push_str(&placeholders.join(", "));
        query.push_str(")");

        // Build the query
        let mut q = sqlx::query_as::<_, Task>(&query);

        // Bind each ID
        for id in ids {
            q = q.bind(id);
        }

        // Execute the query
        Ok(q.fetch_all(&self.base.pool).await?)
    }

    /// Create multiple tasks in a single transaction
    #[instrument(skip(self, tasks))]
    async fn batch_create(&self, tasks: &[Task]) -> Result<Vec<Task>> {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Creating {} tasks in batch", tasks.len());

        // Validate all tasks before starting the transaction
        for task in tasks {
            crate::repositories::validation::ValidationExt::<Task>::validate(self, task)?;
        }

        // Start a transaction
        let mut tx = self.base.pool.begin().await?;

        let mut created_tasks = Vec::with_capacity(tasks.len());

        for task in tasks {
            let now = Utc::now();
            let metadata = task.metadata.as_deref();

            let created_task = sqlx::query_as!(
                Task,
                r#"INSERT INTO tasks (
                    id, title, description, status, start_time, end_time, due_date, priority, importance, tags, url, metadata,
                    created_at, updated_at, created_by_id, assignee_participant_id, workspace_id, conversation_id, memory_id, plan_id, document_id, file_id
                ) VALUES (
                 ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                 ) RETURNING
                    id as "id: _", title, description, status as "status: TaskStatus", start_time as "start_time: _",
                    end_time as "end_time: _", due_date as "due_date: _", priority as "priority: TaskPriority",
                    importance as "importance: TaskImportance", tags as "tags: _", url, metadata as "metadata: _",
                    created_at as "created_at: _", updated_at as "updated_at: _", created_by_id as "created_by_id: _",
                    assignee_participant_id as "assignee_participant_id: _", workspace_id as "workspace_id: _",
                    conversation_id as "conversation_id: _", memory_id as "memory_id: _", plan_id as "plan_id: _",
                    document_id as "document_id: _", file_id as "file_id: _""#,
                task.id,
                task.title,
                task.description,
                task.status,
                task.start_time,
                task.end_time,
                task.due_date,
                task.priority,
                task.importance,
                task.tags,
                task.url,
                metadata,
                now,
                now,
                task.created_by_id,
                task.assignee_participant_id,
                task.workspace_id,
                task.conversation_id,
                task.memory_id,
                task.plan_id,
                task.document_id,
                task.file_id
            )
            .fetch_one(&mut *tx)
            .await?;

            created_tasks.push(created_task);
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(created_tasks)
    }

    /// Update multiple tasks in a single transaction
    #[instrument(skip(self, tasks))]
    async fn batch_update(&self, tasks: &[Task]) -> Result<()> {
        if tasks.is_empty() {
            return Ok(());
        }

        debug!("Updating {} tasks in batch", tasks.len());

        // Validate all tasks before starting the transaction
        for task in tasks {
            crate::repositories::validation::ValidationExt::<Task>::validate(self, task)?;
        }

        // Start a transaction
        let mut tx = self.base.pool.begin().await?;

        for task in tasks {
            let now = Utc::now();
            let metadata = task.metadata.as_deref();

            let affected = sqlx::query!( 
                r#"UPDATE tasks SET title = ?, description = ?, status = ?,
                    start_time = ?, end_time = ?, due_date = ?, priority = ?, importance = ?,
                    tags = ?, url = ?, metadata = ?, updated_at = ?,
                    created_by_id = ?, assignee_participant_id = ?, workspace_id = ?,
                    conversation_id = ?, memory_id = ?, plan_id = ?, document_id = ?, file_id = ?
                WHERE id = ?"#,
                task.title,
                task.description,
                task.status,
                task.start_time,
                task.end_time,
                task.due_date,
                task.priority,
                task.importance,
                task.tags,
                task.url,
                metadata,
                now,
                task.created_by_id,
                task.assignee_participant_id,
                task.workspace_id,
                task.conversation_id,
                task.memory_id,
                task.plan_id,
                task.document_id,
                task.file_id,
                task.id
            )
            .execute(&mut *tx)
            .await?;

            if affected.rows_affected() == 0 {
                tx.rollback().await?;
                return Err(AppError::not_found("Task", task.id));
            }
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }

    /// Delete multiple tasks in a single transaction
    #[instrument(skip(self))]
    async fn batch_delete(&self, ids: &[Uuid]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        debug!("Deleting {} tasks in batch", ids.len());

        // Start a transaction
        let mut tx = self.base.pool.begin().await?;

        // Build a query with IN clause
        let mut query = "DELETE FROM tasks WHERE id IN (".to_string();

        // Add placeholders for each ID
        let placeholders: Vec<String> = (0..ids.len()).map(|i| format!("${}", i + 1)).collect();
        query.push_str(&placeholders.join(", "));
        query.push_str(")");

        // Build the query
        let mut q = sqlx::query(&query);

        // Bind each ID
        for id in ids {
            q = q.bind(id);
        }

        // Execute the query
        let result = q.execute(&mut *tx).await?;

        // Check if any rows were affected
        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::not_found("Task", "batch delete - no tasks found"));
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }
    #[instrument(skip(self))]
    async fn get_by_id(&self, id: &Uuid) -> Result<Option<Task>> {
        debug!("Getting task by ID: {}", id);
        Ok(sqlx::query_as!( 
            Task,
            r#"SELECT 
                id as "id: _", title, description, status as "status: TaskStatus", start_time as "start_time: _",
                end_time as "end_time: _", due_date as "due_date: _", priority as "priority: TaskPriority",
                importance as "importance: TaskImportance", tags as "tags: _", url, metadata as "metadata: _",
                created_at as "created_at: _", updated_at as "updated_at: _", created_by_id as "created_by_id: _",
                assignee_participant_id as "assignee_participant_id: _", workspace_id as "workspace_id: _",
                conversation_id as "conversation_id: _", memory_id as "memory_id: _", plan_id as "plan_id: _",
                document_id as "document_id: _", file_id as "file_id: _"
            FROM tasks WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.base.pool)
        .await?)
    }

    #[instrument(skip(self))]
    async fn list(&self, filter: &TaskFilter) -> Result<Vec<Task>> {
        debug!("Listing tasks with filter: {:?}", filter);

        let mut query_builder = TaskQueryBuilder::new();

        // Apply filters
        query_builder
            .with_workspace_id(filter.workspace_id)
            .with_plan_id(filter.plan_id)
            .with_status(filter.status)
            .with_priority(filter.priority)
            .with_importance(filter.importance)
            .active_only(filter.active_only)
            .overdue_only(filter.overdue_only)
            .with_due_date_range(filter.due_date_after, filter.due_date_before)
            .with_search_term(filter.search_term.as_deref())
            .with_pagination(filter.limit, filter.offset)
            .with_default_ordering();

        // Execute query
        Ok(query_builder
            .build_query_as_task()
            .fetch_all(&self.base.pool)
            .await?)
    }

    #[instrument(skip(self))]
    async fn create(&self, task: &Task) -> Result<Task> {
        // Validate the task before creating it
        crate::repositories::validation::ValidationExt::<Task>::validate(self, task)?;

        let id = Uuid::new_v4();
        debug!("Creating task with ID: {}", id);
        let metadata = task.metadata.as_deref();
        let now = Utc::now();

        Ok(sqlx::query_as!(
            Task,
            r#"INSERT INTO tasks (
                id, title, description, status, start_time, end_time, due_date, priority, importance, tags, url, metadata,
                created_at, updated_at, created_by_id, assignee_participant_id, workspace_id, conversation_id, memory_id, plan_id, document_id, file_id
            ) VALUES (
             ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
             ) RETURNING
                id as "id: _", title, description, status as "status: TaskStatus", start_time as "start_time: _",
                end_time as "end_time: _", due_date as "due_date: _", priority as "priority: TaskPriority",
                importance as "importance: TaskImportance", tags as "tags: _", url, metadata as "metadata: _",
                created_at as "created_at: _", updated_at as "updated_at: _", created_by_id as "created_by_id: _",
                assignee_participant_id as "assignee_participant_id: _", workspace_id as "workspace_id: _",
                conversation_id as "conversation_id: _", memory_id as "memory_id: _", plan_id as "plan_id: _",
                document_id as "document_id: _", file_id as "file_id: _""#,
            id,
            task.title,
            task.description,
            task.status,
            task.start_time,
            task.end_time,
            task.due_date,
            task.priority,
            task.importance,
            task.tags,
            task.url,
            metadata,
            now,
            now,
            task.created_by_id,
            task.assignee_participant_id,
            task.workspace_id,
            task.conversation_id,
            task.memory_id,
            task.plan_id,
            task.document_id,
            task.file_id
        )
        .fetch_one(&self.base.pool)
        .await?)
    }

    #[instrument(skip(self))]
    async fn update(&self, task: &Task) -> Result<()> {
        // Validate the task before updating it
        crate::repositories::validation::ValidationExt::<Task>::validate(self, task)?;

        debug!("Updating task with ID: {}", task.id);  
        let now = Utc::now();
        let metadata = task.metadata.as_deref();
        let affected = sqlx::query!( 
            r#"UPDATE tasks SET title = ?, description = ?, status = ?,
                start_time = ?, end_time = ?, due_date = ?, priority = ?, importance = ?,
                tags = ?, url = ?, metadata = ?, updated_at = ?,
                created_by_id = ?, assignee_participant_id = ?, workspace_id = ?,
                conversation_id = ?, memory_id = ?, plan_id = ?, document_id = ?, file_id = ?
            WHERE id = ?"#,
            task.title,
            task.description,
            task.status,
            task.start_time,
            task.end_time,
            task.due_date,
            task.priority,
            task.importance,
            task.tags,
            task.url,
            metadata,
            now,
            task.created_by_id,
            task.assignee_participant_id,
            task.workspace_id,
            task.conversation_id,
            task.memory_id,
            task.plan_id,
            task.document_id,
            task.file_id,
            task.id
        )
        .execute(&self.base.pool)
        .await?;

        if affected.rows_affected() == 0 {
            return Err(AppError::not_found("Task", task.id));
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting task with ID: {}", id);
        let affected = sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
            .execute(&self.base.pool)
            .await?;
        if affected.rows_affected() == 0 {
            return Err(AppError::not_found("Task", id));
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn count(&self, filter: &TaskFilter) -> Result<i64> {
        debug!("Counting tasks with filter: {:?}", filter);

        let mut query_builder = TaskQueryBuilder::new();

        // Apply filters
        query_builder
            .with_workspace_id(filter.workspace_id)
            .with_plan_id(filter.plan_id)
            .with_status(filter.status)
            .with_priority(filter.priority)
            .with_importance(filter.importance)
            .active_only(filter.active_only)
            .overdue_only(filter.overdue_only)
            .with_due_date_range(filter.due_date_after, filter.due_date_before)
            .with_search_term(filter.search_term.as_deref());

        // Replace SELECT clause with COUNT(*)
        let count_query = format!("SELECT COUNT(*) as count FROM tasks{}", 
            if query_builder.builder().has_where { 
                format!(" WHERE{}", query_builder.builder().builder().sql().split_once("WHERE").unwrap().1) 
            } else { 
                String::new() 
            }
        );

        // Execute count query
        let count = sqlx::query!(&count_query)
            .fetch_one(&self.base.pool)
            .await?;

        Ok(count.count.unwrap_or(0))
    }
}
