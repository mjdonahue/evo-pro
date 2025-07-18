use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum TaskImportance {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Failed = 3,
}

#[boilermates("CreateTask")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[boilermates(not_in("CreateTask"))]
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: TaskPriority,
    pub importance: TaskImportance,
    pub tags: Json<Value>,
    pub url: Option<String>,
    pub metadata: Option<Json<Value>>, // JSON
    #[boilermates(not_in("CreateTask"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateTask"))]
    pub updated_at: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
    pub assignee_participant_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub memory_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    pub file_id: Option<Uuid>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct TaskFilter {
    pub workspace_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub importance: Option<TaskImportance>,
    pub created_by_id: Option<Uuid>,
    pub assignee_participant_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub memory_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    pub file_id: Option<Uuid>,
    pub active_only: Option<bool>, // Excludes completed and failed
    pub overdue_only: Option<bool>,
    pub due_date_after: Option<DateTime<Utc>>,
    pub due_date_before: Option<DateTime<Utc>>,
    pub search_term: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new task
    #[instrument(skip(self))]
    pub async fn create_task(&self, task: &Task) -> Result<Task> {
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
        .fetch_one(&self.pool)
        .await?)
    }   

    /// Get task by ID
    #[instrument(skip(self))]
    pub async fn get_task_by_id(&self, id: &Uuid) -> Result<Option<Task>> {
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
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List tasks with filtering
    #[instrument(skip(self))]
    pub async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<Task>> {
        debug!("Listing tasks with filter: {:?}", filter);
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT
                id as "id: _", plan_id as "plan_id: _", workspace_id as "workspace_id: _",
                title, description, status as "status: TaskStatus", start_time as "start_time: _",
                end_time as "end_time: _", due_date as "due_date: _", priority as "priority: TaskPriority",
                importance as "importance: TaskImportance", tags as "tags: _", url, metadata as "metadata: _",
                status as "status: TaskStatus", metadata as "metadata: _", conversation_id as "conversation_id: _",
                memory_id as "memory_id: _", memory_type as "memory_type: MemoryType", document_id as "document_id: _",
                file_id as "file_id: _", url, primary_assignee_id as "primary_assignee_id: _",
                created_by_id as "created_by_id: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM tasks"#,
        );

        let mut add_where = add_where();

        if let Some(plan_id) = &filter.plan_id {
            add_where(&mut qb);
            qb.push("plan_id = ");
            qb.push_bind(plan_id);
        }
        if let Some(workspace_id) = &filter.workspace_id {  
            add_where(&mut qb);
            qb.push("workspace_id = ");
            qb.push_bind(workspace_id);
        }
        if let Some(status) = &filter.status {  
            add_where(&mut qb);
            qb.push("status = ");
            qb.push_bind(status);
        }
        if let Some(priority) = &filter.priority {
            add_where(&mut qb);     
            qb.push("priority = ");
            qb.push_bind(priority);
        }
        if let Some(importance) = &filter.importance {
            add_where(&mut qb);
            qb.push("importance = ");   
            qb.push_bind(importance);
        }
        if let Some(created_by_id) = &filter.created_by_id {
            add_where(&mut qb);
            qb.push("created_by_id = ");
            qb.push_bind(created_by_id);    
        }
        if let Some(assignee_participant_id) = &filter.assignee_participant_id {
            add_where(&mut qb);
            qb.push("assignee_participant_id = ");
            qb.push_bind(assignee_participant_id);
        }       
        if let Some(conversation_id) = &filter.conversation_id {
            add_where(&mut qb);
            qb.push("conversation_id = ");
            qb.push_bind(conversation_id);
        }
        if let Some(memory_id) = &filter.memory_id {    
            add_where(&mut qb);
            qb.push("memory_id = ");
            qb.push_bind(memory_id);
        }
        if let Some(document_id) = &filter.document_id {    
            add_where(&mut qb);
            qb.push("document_id = ");
            qb.push_bind(document_id);
        }
        if let Some(file_id) = &filter.file_id {        
            add_where(&mut qb);
            qb.push("file_id = ");
            qb.push_bind(file_id);
        }

        qb.push(" ORDER BY title ASC");

        if let Some(limit) = &filter.limit {
            add_where(&mut qb);
            qb.push(" LIMIT ");
            qb.push_bind(*limit as i64);
        }
        if let Some(offset) = &filter.offset {
            add_where(&mut qb); 
            qb.push(" OFFSET ");
            qb.push_bind(*offset as i64);
        }
      
        Ok(qb
            .build_query_as::<'_, Task>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update task
    #[instrument(skip(self))]
    pub async fn update_task(&self, task: &Task) -> Result<()> { 
        debug!("Updating task with ID: {}", task.id);  
        let now = Utc::now();
        let metadata = task.metadata.as_deref();
        let affected = sqlx::query!( 
            r#"UPDATE tasks SET title = ?, description = ?, status = ?,
                start_time = ?, end_time = ?, due_date = ?, priority = ?, importance = ?,
                tags = ?, url = ?, metadata = ?, created_at = ?, updated_at = ?,
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
        .execute(&self.pool)
        .await?;

        if affected.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task with ID {} not found", task.id
            )));
        }
        Ok(())
    }

    /// Delete task
    #[instrument(skip(self))]
    pub async fn delete_task(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting task with ID: {}", id);
        let affected = sqlx::query!("DELETE FROM tasks WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        if affected.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task with ID {id} not found"
            )));
        }
        Ok(())
    }

    /// Update task status
    #[instrument(skip(self))]
    pub async fn update_task_status(&self, id: &Uuid, status: TaskStatus) -> Result<()> {
        let affected = sqlx::query!("UPDATE tasks SET status = ? WHERE id = ?", status, id)
            .execute(&self.pool)
            .await?;
        if affected.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task with ID {id} not found"
            )));
        }
        Ok(())
    }

    /// Start task (mark as in progress)
    pub async fn start_task(&self, id: &Uuid) -> Result<()> {
        self.update_task_status(id, TaskStatus::InProgress).await
    }

    /// Complete task
    pub async fn complete_task(&self, id: &Uuid) -> Result<()> {
        self.update_task_status(id, TaskStatus::Completed).await
    }

    /// Fail task
    pub async fn fail_task(&self, id: &Uuid) -> Result<()> {
        self.update_task_status(id, TaskStatus::Failed).await
    }

    /// Get tasks by plan
    #[instrument(skip(self))]
    pub async fn get_tasks_by_plan(&self, plan_id: &Uuid) -> Result<Vec<Task>> {
        let filter = TaskFilter {
            plan_id: Some(*plan_id),
            search_term: None,
            ..Default::default()
        };

        self.list_tasks(&filter).await
    }

    /// Get overdue tasks
    #[instrument(skip(self))]
    pub async fn get_overdue_tasks(&self) -> Result<Vec<Task>> {
        let filter = TaskFilter {
            overdue_only: Some(true),
            search_term: None,
            ..Default::default()
        };

        self.list_tasks(&filter).await
    }

    /// Get high priority tasks
    #[instrument(skip(self))]
    pub async fn get_high_priority_tasks(&self) -> Result<Vec<Task>> {
        let filter = TaskFilter {
            priority: Some(TaskPriority::High),
            active_only: Some(true),
            search_term: None,
            ..Default::default()
        };

        self.list_tasks(&filter).await
    }

    /// Get task statistics
    #[instrument(skip(self))]
    pub async fn get_task_stats(&self) -> Result<()> {
        let affected = sqlx::query!(
            r#"SELECT
                COUNT(CASE WHEN status = 0 THEN 1 END) as pending_tasks,
                COUNT(CASE WHEN status = 1 THEN 1 END) as in_progress_tasks,
                COUNT(CASE WHEN status = 2 THEN 1 END) as completed_tasks,
                COUNT(CASE WHEN status = 3 THEN 1 END) as failed_tasks,
                COUNT(CASE WHEN priority = 2 THEN 1 END) as high_priority_tasks,
                COUNT(CASE WHEN importance = 2 THEN 1 END) as important_tasks,
                COUNT(CASE WHEN due_date < datetime('now') AND status NOT IN (2, 3) THEN 1 END) as overdue_tasks,
                COUNT(*) as total_tasks
             FROM tasks"#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(())
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
