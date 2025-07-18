use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub participant_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberFilter {
    pub group_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl GroupMember {
    /// Create a new group member relationship
    pub async fn create(pool: &Pool<Sqlite>, group_member: &GroupMember) -> Result<()> {
        sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (?, ?)")
            .bind(group_member.group_id)
            .bind(group_member.participant_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// List group members with filtering
    pub async fn list(pool: &Pool<Sqlite>, filter: &GroupMemberFilter) -> Result<Vec<GroupMember>> {
        let mut qb: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT group_id, participant_id FROM group_members");

        let mut where_conditions = Vec::new();

        if let Some(group_id) = &filter.group_id {
            where_conditions.push(format!("group_id = '{group_id}'"));
        }

        if let Some(participant_id) = &filter.participant_id {
            where_conditions.push(format!("participant_id = '{participant_id}'"));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
        }

        qb.push(" ORDER BY group_id, participant_id");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows = qb.build().fetch_all(pool).await?;
        let mut group_members = Vec::new();

        for row in rows {
            group_members.push(GroupMember {
                group_id: row
                    .get::<Vec<u8>, _>("group_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                participant_id: row
                    .get::<Vec<u8>, _>("participant_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
            });
        }

        Ok(group_members)
    }

    /// Delete group member relationship
    pub async fn delete(pool: &Pool<Sqlite>, group_id: &Uuid, participant_id: &Uuid) -> Result<()> {
        let affected =
            sqlx::query("DELETE FROM group_members WHERE group_id = ? AND participant_id = ?")
                .bind(group_id)
                .bind(participant_id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Group member relationship not found for group {group_id} and participant {participant_id}"
            )));
        }

        Ok(())
    }

    /// Get participants by group
    pub async fn get_participants_by_group(
        pool: &Pool<Sqlite>,
        group_id: &Uuid,
    ) -> Result<Vec<GroupMember>> {
        let filter = GroupMemberFilter {
            group_id: Some(*group_id),
            participant_id: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get groups by participant
    pub async fn get_groups_by_participant(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<Vec<GroupMember>> {
        let filter = GroupMemberFilter {
            group_id: None,
            participant_id: Some(*participant_id),
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Check if participant is member of group
    pub async fn is_member(
        pool: &Pool<Sqlite>,
        group_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<bool> {
        let count = sqlx::query(
            "SELECT COUNT(*) as count FROM group_members 
             WHERE group_id = ? AND participant_id = ?",
        )
        .bind(group_id)
        .bind(participant_id)
        .fetch_one(pool)
        .await?;

        Ok(count.get::<i32, _>("count") > 0)
    }

    /// Count participants in group
    pub async fn count_participants_in_group(pool: &Pool<Sqlite>, group_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM group_members WHERE group_id = ?")
            .bind(group_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Count groups for participant
    pub async fn count_groups_for_participant(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<i64> {
        let row =
            sqlx::query("SELECT COUNT(*) as count FROM group_members WHERE participant_id = ?")
                .bind(participant_id)
                .fetch_one(pool)
                .await?;

        Ok(row.get("count"))
    }

    /// Delete all participants from a group
    pub async fn delete_all_participants_from_group(
        pool: &Pool<Sqlite>,
        group_id: &Uuid,
    ) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM group_members WHERE group_id = ?")
            .bind(group_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Delete participant from all groups
    pub async fn delete_participant_from_all_groups(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM group_members WHERE participant_id = ?")
            .bind(participant_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Add multiple participants to a group
    pub async fn add_participants_to_group(
        pool: &Pool<Sqlite>,
        group_id: &Uuid,
        participant_ids: &[Uuid],
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        for participant_id in participant_ids {
            sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (?, ?)")
                .bind(group_id)
                .bind(participant_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Add participant to multiple groups
    pub async fn add_participant_to_groups(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
        group_ids: &[Uuid],
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        for group_id in group_ids {
            sqlx::query("INSERT INTO group_members (group_id, participant_id) VALUES (?, ?)")
                .bind(group_id)
                .bind(participant_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Remove multiple participants from a group
    pub async fn remove_participants_from_group(
        pool: &Pool<Sqlite>,
        group_id: &Uuid,
        participant_ids: &[Uuid],
    ) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let mut total_affected = 0u64;

        for participant_id in participant_ids {
            let affected =
                sqlx::query("DELETE FROM group_members WHERE group_id = ? AND participant_id = ?")
                    .bind(group_id)
                    .bind(participant_id)
                    .execute(&mut *tx)
                    .await?
                    .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }

    /// Remove participant from multiple groups
    pub async fn remove_participant_from_groups(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
        group_ids: &[Uuid],
    ) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let mut total_affected = 0u64;

        for group_id in group_ids {
            let affected =
                sqlx::query("DELETE FROM group_members WHERE group_id = ? AND participant_id = ?")
                    .bind(group_id)
                    .bind(participant_id)
                    .execute(&mut *tx)
                    .await?
                    .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }

    /// Get all group member relationships
    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<GroupMember>> {
        let filter = GroupMemberFilter {
            group_id: None,
            participant_id: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get group members with pagination
    pub async fn get_participants_by_group_paginated(
        pool: &Pool<Sqlite>,
        group_id: &Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<GroupMember>> {
        let filter = GroupMemberFilter {
            group_id: Some(*group_id),
            participant_id: None,
            limit: Some(limit),
            offset: Some(offset),
        };

        Self::list(pool, &filter).await
    }

    /// Get groups for participant with pagination
    pub async fn get_groups_by_participant_paginated(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<GroupMember>> {
        let filter = GroupMemberFilter {
            group_id: None,
            participant_id: Some(*participant_id),
            limit: Some(limit),
            offset: Some(offset),
        };

        Self::list(pool, &filter).await
    }

    /// Check if a group is empty (has no members)
    pub async fn is_group_empty(pool: &Pool<Sqlite>, group_id: &Uuid) -> Result<bool> {
        let count = Self::count_participants_in_group(pool, group_id).await?;
        Ok(count == 0)
    }

    /// Get participant IDs for a group
    pub async fn get_participant_ids_for_group(
        pool: &Pool<Sqlite>,
        group_id: &Uuid,
    ) -> Result<Vec<Uuid>> {
        let members = Self::get_participants_by_group(pool, group_id).await?;
        Ok(members.into_iter().map(|m| m.participant_id).collect())
    }

    /// Get group IDs for a participant
    pub async fn get_group_ids_for_participant(
        pool: &Pool<Sqlite>,
        participant_id: &Uuid,
    ) -> Result<Vec<Uuid>> {
        let members = Self::get_groups_by_participant(pool, participant_id).await?;
        Ok(members.into_iter().map(|m| m.group_id).collect())
    }
}
