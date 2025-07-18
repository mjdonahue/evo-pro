use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub group_type: GroupType,
    pub status: GroupStatus,
    pub metadata: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub workspace_id: Uuid,
    pub parent_group_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GroupType {
    Private = 0,
    Public = 1,
    Other = 2,
}

impl TryFrom<i32> for GroupType {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(GroupType::Private),
            1 => Ok(GroupType::Public),
            2 => Ok(GroupType::Other),
            _ => Err(AppError::ValidationError("Invalid group type".to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GroupStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}

impl TryFrom<i32> for GroupStatus {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(GroupStatus::Active),
            1 => Ok(GroupStatus::Archived),
            2 => Ok(GroupStatus::Deleted),
            _ => Err(AppError::ValidationError(
                "Invalid group status".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFilter {
    pub workspace_id: Option<Uuid>,
    pub parent_group_id: Option<Uuid>,
    pub group_type: Option<GroupType>,
    pub status: Option<GroupStatus>,
    pub search_term: Option<String>,
    pub active_only: Option<bool>,
    pub top_level_only: Option<bool>, // Groups without parent
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Group {
    /// Create a new group
    pub async fn create(pool: &Pool<Sqlite>, group: &Group) -> Result<()> {
        sqlx::query(
            "INSERT INTO groups (
                id, name, description, avatar, type, status, metadata,
                created_at, updated_at, workspace_id, parent_group_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(group.id)
        .bind(&group.name)
        .bind(&group.description)
        .bind(&group.avatar)
        .bind(group.group_type as i32)
        .bind(group.status as i32)
        .bind(&group.metadata)
        .bind(group.created_at)
        .bind(group.updated_at)
        .bind(group.workspace_id)
        .bind(group.parent_group_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get group by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &Uuid) -> Result<Option<Group>> {
        let row = sqlx::query(
            "SELECT id, name, description, avatar, type, status, metadata,
                    created_at, updated_at, workspace_id, parent_group_id
             FROM groups WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Group {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                avatar: row.get("avatar"),
                group_type: GroupType::try_from(row.get::<i32, _>("type"))?,
                status: GroupStatus::try_from(row.get::<i32, _>("status"))?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                parent_group_id: row
                    .get::<Option<Vec<u8>>, _>("parent_group_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List groups with filtering
    pub async fn list(pool: &Pool<Sqlite>, filter: &GroupFilter) -> Result<Vec<Group>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, name, description, avatar, type, status, metadata,
                    created_at, updated_at, workspace_id, parent_group_id
             FROM groups",
        );

        let mut where_conditions = Vec::new();

        if let Some(workspace_id) = &filter.workspace_id {
            where_conditions.push(format!("workspace_id = '{workspace_id}'"));
        }

        if let Some(parent_group_id) = &filter.parent_group_id {
            where_conditions.push(format!("parent_group_id = '{parent_group_id}'"));
        }

        if let Some(group_type) = filter.group_type {
            where_conditions.push(format!("type = {}", group_type as i32));
        }

        if let Some(status) = filter.status {
            where_conditions.push(format!("status = {}", status as i32));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(name LIKE '%{search_term}%' OR description LIKE '%{search_term}%')"
            ));
        }

        if filter.active_only.unwrap_or(false) {
            where_conditions.push("status = 0".to_string()); // Active status
        }

        if filter.top_level_only.unwrap_or(false) {
            where_conditions.push("parent_group_id IS NULL".to_string());
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
        let mut groups = Vec::new();

        for row in rows {
            groups.push(Group {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                avatar: row.get("avatar"),
                group_type: GroupType::try_from(row.get::<i32, _>("type"))?,
                status: GroupStatus::try_from(row.get::<i32, _>("status"))?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                parent_group_id: row
                    .get::<Option<Vec<u8>>, _>("parent_group_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
            });
        }

        Ok(groups)
    }

    /// Update group
    pub async fn update(pool: &Pool<Sqlite>, group: &Group) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE groups SET
                name = ?, description = ?, avatar = ?, type = ?, status = ?,
                metadata = ?, updated_at = ?, workspace_id = ?, parent_group_id = ?
             WHERE id = ?",
        )
        .bind(&group.name)
        .bind(&group.description)
        .bind(&group.avatar)
        .bind(group.group_type as i32)
        .bind(group.status as i32)
        .bind(&group.metadata)
        .bind(group.updated_at)
        .bind(group.workspace_id)
        .bind(group.parent_group_id)
        .bind(group.id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Group with ID {} not found",
                group.id
            )));
        }

        Ok(())
    }

    /// Delete group
    pub async fn delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let affected = sqlx::query("DELETE FROM groups WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Group with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get groups by workspace
    pub async fn get_by_workspace(pool: &Pool<Sqlite>, workspace_id: &Uuid) -> Result<Vec<Group>> {
        let filter = GroupFilter {
            workspace_id: Some(*workspace_id),
            parent_group_id: None,
            group_type: None,
            status: None,
            search_term: None,
            active_only: None,
            top_level_only: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get child groups
    pub async fn get_children(pool: &Pool<Sqlite>, parent_id: &Uuid) -> Result<Vec<Group>> {
        let filter = GroupFilter {
            workspace_id: None,
            parent_group_id: Some(*parent_id),
            group_type: None,
            status: None,
            search_term: None,
            active_only: None,
            top_level_only: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get top-level groups (no parent)
    pub async fn get_top_level(pool: &Pool<Sqlite>, workspace_id: &Uuid) -> Result<Vec<Group>> {
        let filter = GroupFilter {
            workspace_id: Some(*workspace_id),
            parent_group_id: None,
            group_type: None,
            status: None,
            search_term: None,
            active_only: None,
            top_level_only: Some(true),
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get active groups by type
    pub async fn get_active_by_type(
        pool: &Pool<Sqlite>,
        workspace_id: &Uuid,
        group_type: GroupType,
    ) -> Result<Vec<Group>> {
        let filter = GroupFilter {
            workspace_id: Some(*workspace_id),
            parent_group_id: None,
            group_type: Some(group_type),
            status: Some(GroupStatus::Active),
            search_term: None,
            active_only: Some(true),
            top_level_only: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Update group status
    pub async fn update_status(pool: &Pool<Sqlite>, id: &Uuid, status: GroupStatus) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query("UPDATE groups SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status as i32)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Group with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Archive group
    pub async fn archive(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, GroupStatus::Archived).await
    }

    /// Restore archived group
    pub async fn restore(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, GroupStatus::Active).await
    }

    /// Soft delete group
    pub async fn soft_delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, GroupStatus::Deleted).await
    }

    /// Update group metadata
    pub async fn update_metadata(
        pool: &Pool<Sqlite>,
        id: &Uuid,
        metadata: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query("UPDATE groups SET metadata = ?, updated_at = ? WHERE id = ?")
            .bind(metadata)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Group with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Move group to new parent
    pub async fn move_to_parent(
        pool: &Pool<Sqlite>,
        id: &Uuid,
        new_parent_id: Option<&Uuid>,
    ) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE groups SET parent_group_id = ?, updated_at = ? WHERE id = ?")
                .bind(new_parent_id)
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Group with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Search groups by name
    pub async fn search_by_name(
        pool: &Pool<Sqlite>,
        workspace_id: &Uuid,
        search_term: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Group>> {
        let filter = GroupFilter {
            workspace_id: Some(*workspace_id),
            parent_group_id: None,
            group_type: None,
            status: None,
            search_term: Some(search_term.to_string()),
            active_only: None,
            top_level_only: None,
            limit,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Count groups by workspace
    pub async fn count_by_workspace(pool: &Pool<Sqlite>, workspace_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM groups WHERE workspace_id = ?")
            .bind(workspace_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Count child groups
    pub async fn count_children(pool: &Pool<Sqlite>, parent_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM groups WHERE parent_group_id = ?")
            .bind(parent_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Check if group has children
    pub async fn has_children(pool: &Pool<Sqlite>, id: &Uuid) -> Result<bool> {
        let count = Self::count_children(pool, id).await?;
        Ok(count > 0)
    }

    /// Get all descendants (recursive)
    pub async fn get_descendants(pool: &Pool<Sqlite>, parent_id: &Uuid) -> Result<Vec<Group>> {
        let mut descendants = Vec::new();
        let mut to_process = vec![*parent_id];

        while let Some(current_id) = to_process.pop() {
            let children = Self::get_children(pool, &current_id).await?;
            for child in children {
                to_process.push(child.id);
                descendants.push(child);
            }
        }

        Ok(descendants)
    }

    /// Delete group and all children (cascade)
    pub async fn delete_cascade(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let mut tx = pool.begin().await?;

        // First get all descendants
        let descendants = Self::get_descendants(pool, id).await?;

        // Delete all descendants
        for descendant in descendants {
            sqlx::query("DELETE FROM groups WHERE id = ?")
                .bind(descendant.id)
                .execute(&mut *tx)
                .await?;
        }

        // Delete the parent group
        let affected = sqlx::query("DELETE FROM groups WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?
            .rows_affected();

        if affected == 0 {
            tx.rollback().await?;
            return Err(AppError::NotFoundError(format!(
                "Group with ID {id} not found"
            )));
        }

        tx.commit().await?;
        Ok(())
    }
}
