use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignee {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub task_id: Uuid,
    pub participant_id: Uuid,
    pub role: TaskAssigneeRole,
    pub status: TaskAssigneeStatus,
    pub metadata: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskAssigneeRole {
    Primary,
    Secondary,
    Other,
}

impl TryFrom<String> for TaskAssigneeRole {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "primary" => Ok(TaskAssigneeRole::Primary),
            "secondary" => Ok(TaskAssigneeRole::Secondary),
            "other" => Ok(TaskAssigneeRole::Other),
            _ => Err(AppError::ValidationError(
                "Invalid task assignee role".to_string(),
            )),
        }
    }
}

impl ToString for TaskAssigneeRole {
    fn to_string(&self) -> String {
        match self {
            TaskAssigneeRole::Primary => "primary".to_string(),
            TaskAssigneeRole::Secondary => "secondary".to_string(),
            TaskAssigneeRole::Other => "other".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskAssigneeStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl TryFrom<String> for TaskAssigneeStatus {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "pending" => Ok(TaskAssigneeStatus::Pending),
            "in_progress" => Ok(TaskAssigneeStatus::InProgress),
            "completed" => Ok(TaskAssigneeStatus::Completed),
            "failed" => Ok(TaskAssigneeStatus::Failed),
            _ => Err(AppError::ValidationError(
                "Invalid task assignee status".to_string(),
            )),
        }
    }
}

impl ToString for TaskAssigneeStatus {
    fn to_string(&self) -> String {
        match self {
            TaskAssigneeStatus::Pending => "pending".to_string(),
            TaskAssigneeStatus::InProgress => "in_progress".to_string(),
            TaskAssigneeStatus::Completed => "completed".to_string(),
            TaskAssigneeStatus::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssigneeFilter {
    pub workspace_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub role: Option<TaskAssigneeRole>,
    pub status: Option<TaskAssigneeStatus>,
    pub primary_only: Option<bool>,
    pub active_only: Option<bool>, // Excludes completed and failed
    pub completed_only: Option<bool>,
    pub failed_only: Option<bool>,
    pub pending_only: Option<bool>,
    pub in_progress_only: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl TaskAssignee {
    /// Create a new task assignee
    pub async fn create(pool: &Pool<Sqlite>, task_assignee: &TaskAssignee) -> Result<()> {
        sqlx::query(
            "INSERT INTO task_assignees (
                id, workspace_id, task_id, participant_id, role, status,
                metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(task_assignee.id)
        .bind(task_assignee.workspace_id)
        .bind(task_assignee.task_id)
        .bind(task_assignee.participant_id)
        .bind(task_assignee.role.to_string())
        .bind(task_assignee.status.to_string())
        .bind(&task_assignee.metadata)
        .bind(task_assignee.created_at)
        .bind(task_assignee.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get task assignee by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &Uuid) -> Result<Option<TaskAssignee>> {
        let row = sqlx::query(
            "SELECT id, workspace_id, task_id, participant_id, role, status,
                    metadata, created_at, updated_at
             FROM task_assignees WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(TaskAssignee {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                task_id: row
                    .get::<Vec<u8>, _>("task_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                participant_id: row
                    .get::<Vec<u8>, _>("participant_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                role: TaskAssigneeRole::try_from(row.get::<String, _>("role"))?,
                status: TaskAssigneeStatus::try_from(row.get::<String, _>("status"))?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// List task assignees with filtering
    pub async fn list(
        pool: &Pool<Sqlite>,
        filter: &TaskAssigneeFilter,
    ) -> Result<Vec<TaskAssignee>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, workspace_id, task_id, participant_id, role, status,
                    metadata, created_at, updated_at
             FROM task_assignees",
        );

        let mut where_conditions = Vec::new();

        if let Some(workspace_id) = &filter.workspace_id {
            where_conditions.push(format!("workspace_id = '{workspace_id}'"));
        }

        if let Some(task_id) = &filter.task_id {
            where_conditions.push(format!("task_id = '{task_id}'"));
        }

        if let Some(participant_id) = &filter.participant_id {
            where_conditions.push(format!("participant_id = '{participant_id}'"));
        }

        if let Some(role) = filter.role {
            where_conditions.push(format!("role = '{}'", role.to_string()));
        }

        if let Some(status) = filter.status {
            where_conditions.push(format!("status = '{}'", status.to_string()));
        }

        if filter.primary_only.unwrap_or(false) {
            where_conditions.push("role = 'primary'".to_string());
        }

        if filter.active_only.unwrap_or(false) {
            where_conditions.push("status NOT IN ('completed', 'failed')".to_string());
        }

        if filter.completed_only.unwrap_or(false) {
            where_conditions.push("status = 'completed'".to_string());
        }

        if filter.failed_only.unwrap_or(false) {
            where_conditions.push("status = 'failed'".to_string());
        }

        if filter.pending_only.unwrap_or(false) {
            where_conditions.push("status = 'pending'".to_string());
        }

        if filter.in_progress_only.unwrap_or(false) {
            where_conditions.push("status = 'in_progress'".to_string());
        }

        if let Some(created_after) = &filter.created_after {
            where_conditions.push(format!(
                "created_at >= '{}'",
                created_after.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if let Some(created_before) = &filter.created_before {
            where_conditions.push(format!(
                "created_at <= '{}'",
                created_before.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if let Some(updated_after) = &filter.updated_after {
            where_conditions.push(format!(
                "updated_at >= '{}'",
                updated_after.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if let Some(updated_before) = &filter.updated_before {
            where_conditions.push(format!(
                "updated_at <= '{}'",
                updated_before.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
        }

        qb.push(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows = qb.build().fetch_all(pool).await?;
        let mut task_assignees = Vec::new();

        for row in rows {
            task_assignees.push(TaskAssignee {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                task_id: row
                    .get::<Vec<u8>, _>("task_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                participant_id: row
                    .get::<Vec<u8>, _>("participant_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                role: TaskAssigneeRole::try_from(row.get::<String, _>("role"))?,
                status: TaskAssigneeStatus::try_from(row.get::<String, _>("status"))?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(task_assignees)
    }

    /// Update task assignee
    pub async fn update(pool: &Pool<Sqlite>, task_assignee: &TaskAssignee) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE task_assignees SET
                workspace_id = ?, task_id = ?, participant_id = ?, role = ?,
                status = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(task_assignee.workspace_id)
        .bind(task_assignee.task_id)
        .bind(task_assignee.participant_id)
        .bind(task_assignee.role.to_string())
        .bind(task_assignee.status.to_string())
        .bind(&task_assignee.metadata)
        .bind(task_assignee.updated_at)
        .bind(task_assignee.id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task assignee with ID {} not found",
                task_assignee.id
            )));
        }

        Ok(())
    }

    /// Delete task assignee
    pub async fn delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let affected = sqlx::query("DELETE FROM task_assignees WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task assignee with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get task assignees by task
    pub async fn get_by_task(pool: &Pool<Sqlite>, task_id: &Uuid) -> Result<Vec<TaskAssignee>> {
        let filter = TaskAssigneeFilter {
            workspace_id: None,
            task_id: Some(*task_id),
            participant_id: None,
            role: None,
            status: None,
            primary_only: None,
            active_only: None,
            completed_only: None,
            failed_only: None,
            pending_only: None,
            in_progress_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get task assignees by participant
    pub async fn get_by_participant(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<Vec<TaskAssignee>> {
        let filter = TaskAssigneeFilter {
            workspace_id: None,
            task_id: None,
            participant_id: Some(*participant_id),
            role: None,
            status: None,
            primary_only: None,
            active_only: None,
            completed_only: None,
            failed_only: None,
            pending_only: None,
            in_progress_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get primary assignee for task
    pub async fn get_primary_assignee(
        pool: &Pool<Sqlite>,
        task_id: &Uuid,
    ) -> Result<Option<TaskAssignee>> {
        let assignees = sqlx::query(
            "SELECT id, workspace_id, task_id, participant_id, role, status,
                    metadata, created_at, updated_at
             FROM task_assignees 
             WHERE task_id = ? AND role = 'primary'
             LIMIT 1",
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = assignees {
            Ok(Some(TaskAssignee {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                task_id: row
                    .get::<Vec<u8>, _>("task_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                participant_id: row
                    .get::<Vec<u8>, _>("participant_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                role: TaskAssigneeRole::try_from(row.get::<String, _>("role"))?,
                status: TaskAssigneeStatus::try_from(row.get::<String, _>("status"))?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update assignee status
    pub async fn update_status(
        pool: &Pool<Sqlite>,
        id: &Uuid,
        status: TaskAssigneeStatus,
    ) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE task_assignees SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status.to_string())
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task assignee with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Start assignee work (mark as in progress)
    pub async fn start_work(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, TaskAssigneeStatus::InProgress).await
    }

    /// Complete assignee work
    pub async fn complete_work(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, TaskAssigneeStatus::Completed).await
    }

    /// Fail assignee work
    pub async fn fail_work(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, TaskAssigneeStatus::Failed).await
    }

    /// Add assignee to task
    pub async fn add_assignee(
        pool: &Pool<Sqlite>,
        workspace_id: &Uuid,
        task_id: &Uuid,
        participant_id: &Uuid,
        role: TaskAssigneeRole,
        metadata: Option<String>,
    ) -> Result<TaskAssignee> {
        let now = Utc::now();
        let assignee = TaskAssignee {
            id: Uuid::new_v4(),
            workspace_id: *workspace_id,
            task_id: *task_id,
            participant_id: *participant_id,
            role,
            status: TaskAssigneeStatus::Pending,
            metadata,
            created_at: now,
            updated_at: now,
        };

        Self::create(pool, &assignee).await?;
        Ok(assignee)
    }

    /// Remove assignee from task
    pub async fn remove_assignee(
        pool: &Pool<Sqlite>,
        task_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<()> {
        let affected =
            sqlx::query("DELETE FROM task_assignees WHERE task_id = ? AND participant_id = ?")
                .bind(task_id)
                .bind(participant_id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task assignee not found for task {task_id} and participant {participant_id}"
            )));
        }

        Ok(())
    }

    /// Check if participant is assigned to task
    pub async fn is_assigned(
        pool: &Pool<Sqlite>,
        task_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<bool> {
        let count = sqlx::query(
            "SELECT COUNT(*) as count FROM task_assignees WHERE task_id = ? AND participant_id = ?",
        )
        .bind(task_id)
        .bind(participant_id)
        .fetch_one(pool)
        .await?;

        Ok(count.get::<i64, _>("count") > 0)
    }

    /// Get assignee count for task
    pub async fn count_by_task(pool: &Pool<Sqlite>, task_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM task_assignees WHERE task_id = ?")
            .bind(task_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Get task count for participant
    pub async fn count_by_participant(pool: &Pool<Sqlite>, participant_id: &Uuid) -> Result<i64> {
        let row =
            sqlx::query("SELECT COUNT(*) as count FROM task_assignees WHERE participant_id = ?")
                .bind(participant_id)
                .fetch_one(pool)
                .await?;

        Ok(row.get("count"))
    }

    /// Get assignee statistics
    pub async fn get_assignee_stats(pool: &Pool<Sqlite>) -> Result<TaskAssigneeStats> {
        let stats_row = sqlx::query(
            "SELECT 
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_count,
                COUNT(CASE WHEN status = 'in_progress' THEN 1 END) as in_progress_count,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_count,
                COUNT(CASE WHEN role = 'primary' THEN 1 END) as primary_assignees,
                COUNT(CASE WHEN role = 'secondary' THEN 1 END) as secondary_assignees,
                COUNT(CASE WHEN role = 'other' THEN 1 END) as other_assignees,
                COUNT(*) as total_assignees
             FROM task_assignees",
        )
        .fetch_one(pool)
        .await?;

        Ok(TaskAssigneeStats {
            pending_assignees: stats_row.get::<i64, _>("pending_count") as u32,
            in_progress_assignees: stats_row.get::<i64, _>("in_progress_count") as u32,
            completed_assignees: stats_row.get::<i64, _>("completed_count") as u32,
            failed_assignees: stats_row.get::<i64, _>("failed_count") as u32,
            primary_assignees: stats_row.get::<i64, _>("primary_assignees") as u32,
            secondary_assignees: stats_row.get::<i64, _>("secondary_assignees") as u32,
            other_assignees: stats_row.get::<i64, _>("other_assignees") as u32,
            total_assignees: stats_row.get::<i64, _>("total_assignees") as u32,
        })
    }

    /// Get assignee statistics for participant
    pub async fn get_participant_assignee_stats(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<TaskAssigneeStats> {
        let stats_row = sqlx::query(
            "SELECT 
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_count,
                COUNT(CASE WHEN status = 'in_progress' THEN 1 END) as in_progress_count,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_count,
                COUNT(CASE WHEN role = 'primary' THEN 1 END) as primary_assignees,
                COUNT(CASE WHEN role = 'secondary' THEN 1 END) as secondary_assignees,
                COUNT(CASE WHEN role = 'other' THEN 1 END) as other_assignees,
                COUNT(*) as total_assignees
             FROM task_assignees WHERE participant_id = ?",
        )
        .bind(participant_id)
        .fetch_one(pool)
        .await?;

        Ok(TaskAssigneeStats {
            pending_assignees: stats_row.get::<i64, _>("pending_count") as u32,
            in_progress_assignees: stats_row.get::<i64, _>("in_progress_count") as u32,
            completed_assignees: stats_row.get::<i64, _>("completed_count") as u32,
            failed_assignees: stats_row.get::<i64, _>("failed_count") as u32,
            primary_assignees: stats_row.get::<i64, _>("primary_assignees") as u32,
            secondary_assignees: stats_row.get::<i64, _>("secondary_assignees") as u32,
            other_assignees: stats_row.get::<i64, _>("other_assignees") as u32,
            total_assignees: stats_row.get::<i64, _>("total_assignees") as u32,
        })
    }

    /// Delete assignees by task
    pub async fn delete_by_task(pool: &Pool<Sqlite>, task_id: &Uuid) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM task_assignees WHERE task_id = ?")
            .bind(task_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Delete assignees by participant
    pub async fn delete_by_participant(pool: &Pool<Sqlite>, participant_id: &Uuid) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM task_assignees WHERE participant_id = ?")
            .bind(participant_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Bulk add assignees
    pub async fn bulk_add_assignees(
        pool: &Pool<Sqlite>,
        workspace_id: &Uuid,
        task_id: &Uuid,
        assignees: &[(Uuid, TaskAssigneeRole)], // (participant_id, role)
    ) -> Result<Vec<TaskAssignee>> {
        let mut tx = pool.begin().await?;
        let mut created_assignees = Vec::new();
        let now = Utc::now();

        for (participant_id, role) in assignees {
            let assignee = TaskAssignee {
                id: Uuid::new_v4(),
                workspace_id: *workspace_id,
                task_id: *task_id,
                participant_id: *participant_id,
                role: *role,
                status: TaskAssigneeStatus::Pending,
                metadata: None,
                created_at: now,
                updated_at: now,
            };

            sqlx::query(
                "INSERT INTO task_assignees (
                    id, workspace_id, task_id, participant_id, role, status,
                    metadata, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(assignee.id)
            .bind(assignee.workspace_id)
            .bind(assignee.task_id)
            .bind(assignee.participant_id)
            .bind(assignee.role.to_string())
            .bind(assignee.status.to_string())
            .bind(&assignee.metadata)
            .bind(assignee.created_at)
            .bind(assignee.updated_at)
            .execute(&mut *tx)
            .await?;

            created_assignees.push(assignee);
        }

        tx.commit().await?;
        Ok(created_assignees)
    }

    /// Bulk update assignee status
    pub async fn bulk_update_status(
        pool: &Pool<Sqlite>,
        assignee_ids: &[Uuid],
        status: TaskAssigneeStatus,
    ) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let now = Utc::now();
        let mut total_affected = 0u64;

        for assignee_id in assignee_ids {
            let affected =
                sqlx::query("UPDATE task_assignees SET status = ?, updated_at = ? WHERE id = ?")
                    .bind(status.to_string())
                    .bind(now)
                    .bind(assignee_id)
                    .execute(&mut *tx)
                    .await?
                    .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }

    /// Update assignee role
    pub async fn update_role(pool: &Pool<Sqlite>, id: &Uuid, role: TaskAssigneeRole) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE task_assignees SET role = ?, updated_at = ? WHERE id = ?")
                .bind(role.to_string())
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Task assignee with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Transfer primary assignee role
    pub async fn transfer_primary_role(
        pool: &Pool<Sqlite>,
        task_id: &Uuid,
        new_primary_participant_id: &Uuid,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;
        let now = Utc::now();

        // Remove primary role from current assignee
        sqlx::query(
            "UPDATE task_assignees SET role = 'secondary', updated_at = ? WHERE task_id = ? AND role = 'primary'"
        )
        .bind(now)
        .bind(task_id)
        .execute(&mut *tx)
        .await?;

        // Assign primary role to new participant
        let affected = sqlx::query(
            "UPDATE task_assignees SET role = 'primary', updated_at = ? WHERE task_id = ? AND participant_id = ?"
        )
        .bind(now)
        .bind(task_id)
        .bind(new_primary_participant_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if affected == 0 {
            tx.rollback().await?;
            return Err(AppError::NotFoundError(format!(
                "Task assignee not found for task {task_id} and participant {new_primary_participant_id}"
            )));
        }

        tx.commit().await?;
        Ok(())
    }

    /// Get active assignees (not completed or failed)
    pub async fn get_active_assignees(
        pool: &Pool<Sqlite>,
        task_id: &Uuid,
    ) -> Result<Vec<TaskAssignee>> {
        let filter = TaskAssigneeFilter {
            workspace_id: None,
            task_id: Some(*task_id),
            participant_id: None,
            role: None,
            status: None,
            primary_only: None,
            active_only: Some(true),
            completed_only: None,
            failed_only: None,
            pending_only: None,
            in_progress_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssigneeStats {
    pub pending_assignees: u32,
    pub in_progress_assignees: u32,
    pub completed_assignees: u32,
    pub failed_assignees: u32,
    pub primary_assignees: u32,
    pub secondary_assignees: u32,
    pub other_assignees: u32,
    pub total_assignees: u32,
}
