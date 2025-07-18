use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub plan_type: PlanType,
    pub plan_status: PlanStatus,
    pub plan_metadata: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlanType {
    Task,
    Goal,
    Other,
}

impl TryFrom<String> for PlanType {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "task" => Ok(PlanType::Task),
            "goal" => Ok(PlanType::Goal),
            "other" => Ok(PlanType::Other),
            _ => Err(AppError::ValidationError("Invalid plan type".to_string())),
        }
    }
}

impl ToString for PlanType {
    fn to_string(&self) -> String {
        match self {
            PlanType::Task => "task".to_string(),
            PlanType::Goal => "goal".to_string(),
            PlanType::Other => "other".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl TryFrom<String> for PlanStatus {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "pending" => Ok(PlanStatus::Pending),
            "in_progress" => Ok(PlanStatus::InProgress),
            "completed" => Ok(PlanStatus::Completed),
            "failed" => Ok(PlanStatus::Failed),
            _ => Err(AppError::ValidationError("Invalid plan status".to_string())),
        }
    }
}

impl ToString for PlanStatus {
    fn to_string(&self) -> String {
        match self {
            PlanStatus::Pending => "pending".to_string(),
            PlanStatus::InProgress => "in_progress".to_string(),
            PlanStatus::Completed => "completed".to_string(),
            PlanStatus::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFilter {
    pub participant_id: Option<Uuid>,
    pub plan_type: Option<PlanType>,
    pub plan_status: Option<PlanStatus>,
    pub active_only: Option<bool>,     // Excludes completed and failed
    pub incomplete_only: Option<bool>, // Excludes completed
    pub completed_only: Option<bool>,
    pub failed_only: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Plan {
    /// Create a new plan
    pub async fn create(pool: &Pool<Sqlite>, plan: &Plan) -> Result<()> {
        sqlx::query(
            "INSERT INTO plans (
                id, participant_id, plan_type, plan_status, plan_metadata,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(plan.id)
        .bind(plan.participant_id)
        .bind(plan.plan_type.to_string())
        .bind(plan.plan_status.to_string())
        .bind(&plan.plan_metadata)
        .bind(plan.created_at)
        .bind(plan.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get plan by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &Uuid) -> Result<Option<Plan>> {
        let row = sqlx::query(
            "SELECT id, participant_id, plan_type, plan_status, plan_metadata,
                    created_at, updated_at
             FROM plans WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Plan {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                participant_id: row
                    .get::<Vec<u8>, _>("participant_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                plan_type: PlanType::try_from(row.get::<String, _>("plan_type"))?,
                plan_status: PlanStatus::try_from(row.get::<String, _>("plan_status"))?,
                plan_metadata: row.get("plan_metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// List plans with filtering
    pub async fn list(pool: &Pool<Sqlite>, filter: &PlanFilter) -> Result<Vec<Plan>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, participant_id, plan_type, plan_status, plan_metadata,
                    created_at, updated_at
             FROM plans",
        );

        let mut where_conditions = Vec::new();

        if let Some(participant_id) = &filter.participant_id {
            where_conditions.push(format!("participant_id = '{participant_id}'"));
        }

        if let Some(plan_type) = filter.plan_type {
            where_conditions.push(format!("plan_type = '{}'", plan_type.to_string()));
        }

        if let Some(plan_status) = filter.plan_status {
            where_conditions.push(format!("plan_status = '{}'", plan_status.to_string()));
        }

        if filter.active_only.unwrap_or(false) {
            where_conditions.push("plan_status NOT IN ('completed', 'failed')".to_string());
        }

        if filter.incomplete_only.unwrap_or(false) {
            where_conditions.push("plan_status != 'completed'".to_string());
        }

        if filter.completed_only.unwrap_or(false) {
            where_conditions.push("plan_status = 'completed'".to_string());
        }

        if filter.failed_only.unwrap_or(false) {
            where_conditions.push("plan_status = 'failed'".to_string());
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

        qb.push(" ORDER BY updated_at DESC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows = qb.build().fetch_all(pool).await?;
        let mut plans = Vec::new();

        for row in rows {
            plans.push(Plan {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                participant_id: row
                    .get::<Vec<u8>, _>("participant_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                plan_type: PlanType::try_from(row.get::<String, _>("plan_type"))?,
                plan_status: PlanStatus::try_from(row.get::<String, _>("plan_status"))?,
                plan_metadata: row.get("plan_metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(plans)
    }

    /// Update plan
    pub async fn update(pool: &Pool<Sqlite>, plan: &Plan) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE plans SET
                participant_id = ?, plan_type = ?, plan_status = ?, plan_metadata = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(plan.participant_id)
        .bind(plan.plan_type.to_string())
        .bind(plan.plan_status.to_string())
        .bind(&plan.plan_metadata)
        .bind(plan.updated_at)
        .bind(plan.id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Plan with ID {} not found",
                plan.id
            )));
        }

        Ok(())
    }

    /// Delete plan
    pub async fn delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let affected = sqlx::query("DELETE FROM plans WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Plan with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get plans by participant
    pub async fn get_by_participant(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<Vec<Plan>> {
        let filter = PlanFilter {
            participant_id: Some(*participant_id),
            plan_type: None,
            plan_status: None,
            active_only: None,
            incomplete_only: None,
            completed_only: None,
            failed_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get active plans by participant
    pub async fn get_active_by_participant(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<Vec<Plan>> {
        let filter = PlanFilter {
            participant_id: Some(*participant_id),
            plan_type: None,
            plan_status: None,
            active_only: Some(true),
            incomplete_only: None,
            completed_only: None,
            failed_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get plans by type
    pub async fn get_by_type(pool: &Pool<Sqlite>, plan_type: PlanType) -> Result<Vec<Plan>> {
        let filter = PlanFilter {
            participant_id: None,
            plan_type: Some(plan_type),
            plan_status: None,
            active_only: None,
            incomplete_only: None,
            completed_only: None,
            failed_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get plans by status
    pub async fn get_by_status(pool: &Pool<Sqlite>, plan_status: PlanStatus) -> Result<Vec<Plan>> {
        let filter = PlanFilter {
            participant_id: None,
            plan_type: None,
            plan_status: Some(plan_status),
            active_only: None,
            incomplete_only: None,
            completed_only: None,
            failed_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Update plan status
    pub async fn update_status(pool: &Pool<Sqlite>, id: &Uuid, status: PlanStatus) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query("UPDATE plans SET plan_status = ?, updated_at = ? WHERE id = ?")
            .bind(status.to_string())
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Plan with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Mark plan as in progress
    pub async fn start_plan(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, PlanStatus::InProgress).await
    }

    /// Mark plan as completed
    pub async fn complete_plan(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, PlanStatus::Completed).await
    }

    /// Mark plan as failed
    pub async fn fail_plan(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, PlanStatus::Failed).await
    }

    /// Update plan metadata
    pub async fn update_metadata(
        pool: &Pool<Sqlite>,
        id: &Uuid,
        metadata: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE plans SET plan_metadata = ?, updated_at = ? WHERE id = ?")
                .bind(&metadata)
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Plan with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Count plans by participant
    pub async fn count_by_participant(pool: &Pool<Sqlite>, participant_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM plans WHERE participant_id = ?")
            .bind(participant_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Count plans by status
    pub async fn count_by_status(pool: &Pool<Sqlite>, status: PlanStatus) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM plans WHERE plan_status = ?")
            .bind(status.to_string())
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Count plans by type
    pub async fn count_by_type(pool: &Pool<Sqlite>, plan_type: PlanType) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM plans WHERE plan_type = ?")
            .bind(plan_type.to_string())
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Get plan statistics
    pub async fn get_plan_stats(pool: &Pool<Sqlite>) -> Result<PlanStats> {
        let stats_row = sqlx::query(
            "SELECT 
                COUNT(CASE WHEN plan_status = 'pending' THEN 1 END) as pending_count,
                COUNT(CASE WHEN plan_status = 'in_progress' THEN 1 END) as in_progress_count,
                COUNT(CASE WHEN plan_status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN plan_status = 'failed' THEN 1 END) as failed_count,
                COUNT(CASE WHEN plan_type = 'task' THEN 1 END) as task_plans,
                COUNT(CASE WHEN plan_type = 'goal' THEN 1 END) as goal_plans,
                COUNT(CASE WHEN plan_type = 'other' THEN 1 END) as other_plans,
                COUNT(*) as total_plans
             FROM plans",
        )
        .fetch_one(pool)
        .await?;

        Ok(PlanStats {
            pending_plans: stats_row.get::<i64, _>("pending_count") as u32,
            in_progress_plans: stats_row.get::<i64, _>("in_progress_count") as u32,
            completed_plans: stats_row.get::<i64, _>("completed_count") as u32,
            failed_plans: stats_row.get::<i64, _>("failed_count") as u32,
            task_plans: stats_row.get::<i64, _>("task_plans") as u32,
            goal_plans: stats_row.get::<i64, _>("goal_plans") as u32,
            other_plans: stats_row.get::<i64, _>("other_plans") as u32,
            total_plans: stats_row.get::<i64, _>("total_plans") as u32,
        })
    }

    /// Get plan statistics for participant
    pub async fn get_participant_plan_stats(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<PlanStats> {
        let stats_row = sqlx::query(
            "SELECT 
                COUNT(CASE WHEN plan_status = 'pending' THEN 1 END) as pending_count,
                COUNT(CASE WHEN plan_status = 'in_progress' THEN 1 END) as in_progress_count,
                COUNT(CASE WHEN plan_status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN plan_status = 'failed' THEN 1 END) as failed_count,
                COUNT(CASE WHEN plan_type = 'task' THEN 1 END) as task_plans,
                COUNT(CASE WHEN plan_type = 'goal' THEN 1 END) as goal_plans,
                COUNT(CASE WHEN plan_type = 'other' THEN 1 END) as other_plans,
                COUNT(*) as total_plans
             FROM plans WHERE participant_id = ?",
        )
        .bind(participant_id)
        .fetch_one(pool)
        .await?;

        Ok(PlanStats {
            pending_plans: stats_row.get::<i64, _>("pending_count") as u32,
            in_progress_plans: stats_row.get::<i64, _>("in_progress_count") as u32,
            completed_plans: stats_row.get::<i64, _>("completed_count") as u32,
            failed_plans: stats_row.get::<i64, _>("failed_count") as u32,
            task_plans: stats_row.get::<i64, _>("task_plans") as u32,
            goal_plans: stats_row.get::<i64, _>("goal_plans") as u32,
            other_plans: stats_row.get::<i64, _>("other_plans") as u32,
            total_plans: stats_row.get::<i64, _>("total_plans") as u32,
        })
    }

    /// Delete plans by participant
    pub async fn delete_by_participant(pool: &Pool<Sqlite>, participant_id: &Uuid) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM plans WHERE participant_id = ?")
            .bind(participant_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Bulk update plan status
    pub async fn bulk_update_status(
        pool: &Pool<Sqlite>,
        plan_ids: &[Uuid],
        status: PlanStatus,
    ) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let now = Utc::now();
        let mut total_affected = 0u64;

        for plan_id in plan_ids {
            let affected =
                sqlx::query("UPDATE plans SET plan_status = ?, updated_at = ? WHERE id = ?")
                    .bind(status.to_string())
                    .bind(now)
                    .bind(plan_id)
                    .execute(&mut *tx)
                    .await?
                    .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }

    /// Get recently updated plans
    pub async fn get_recently_updated(pool: &Pool<Sqlite>, limit: u32) -> Result<Vec<Plan>> {
        let filter = PlanFilter {
            participant_id: None,
            plan_type: None,
            plan_status: None,
            active_only: None,
            incomplete_only: None,
            completed_only: None,
            failed_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
            limit: Some(limit),
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get overdue plans (in progress but past a certain threshold)
    pub async fn get_overdue_plans(pool: &Pool<Sqlite>, threshold_days: i64) -> Result<Vec<Plan>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(threshold_days);

        let filter = PlanFilter {
            participant_id: None,
            plan_type: None,
            plan_status: Some(PlanStatus::InProgress),
            active_only: None,
            incomplete_only: None,
            completed_only: None,
            failed_only: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: Some(cutoff_date),
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStats {
    pub pending_plans: u32,
    pub in_progress_plans: u32,
    pub completed_plans: u32,
    pub failed_plans: u32,
    pub task_plans: u32,
    pub goal_plans: u32,
    pub other_plans: u32,
    pub total_plans: u32,
}
