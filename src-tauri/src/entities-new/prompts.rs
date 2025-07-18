use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum PromptType {
    System = 0,
    User = 1,
    Agent = 2,
    Tool = 3,
    Other = 4,
}

impl TryFrom<i32> for PromptType {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(PromptType::System),
            1 => Ok(PromptType::User),
            2 => Ok(PromptType::Agent),
            3 => Ok(PromptType::Tool),
            4 => Ok(PromptType::Other),
            _ => Err(AppError::DeserializationError(format!(
                "Invalid PromptType: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum PromptStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}

impl TryFrom<i32> for PromptStatus {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(PromptStatus::Active),
            1 => Ok(PromptStatus::Archived),
            2 => Ok(PromptStatus::Deleted),
            _ => Err(AppError::DeserializationError(format!(
                "Invalid PromptStatus: {value}"
            ))),
        }
    }
}

/// Prompt template structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptTemplate {
    pub name: String,
    pub content: String,
}

impl PromptTemplate {
    pub fn new(name: String, content: String) -> Self {
        Self { name, content }
    }
}

/// Prompt variable structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptVariable {
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
}

impl PromptVariable {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            default_value: None,
            required: false,
        }
    }
}

/// Collection of prompt variables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptVariables {
    pub variables: Vec<PromptVariable>,
}

impl PromptVariables {
    pub fn new(variables: Vec<PromptVariable>) -> Self {
        Self { variables }
    }
}

/// Prompt tag structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptTag {
    pub name: String,
}

impl PromptTag {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

/// Collection of prompt tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptTags {
    pub tags: Vec<PromptTag>,
}

impl PromptTags {
    pub fn new(tags: Vec<PromptTag>) -> Self {
        Self { tags }
    }
}

/// Prompt category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptCategory {
    System,
    User,
    Custom,
}

/// Prompt model matching the SQLite schema
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    pub workspace_id: Uuid,
    pub type_: PromptType,
    pub status: PromptStatus,
    pub metadata: String,
    pub template: String,  // JSON serialized PromptTemplate
    pub variables: String, // JSON serialized PromptVariables
    pub tags: String,      // JSON serialized PromptTags
    pub category: String,  // JSON serialized PromptCategory
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub is_public: bool,
    pub is_system: bool,
    pub is_archived: bool,
    pub is_deleted: bool,
    pub is_featured: bool,
    pub is_verified: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for prompt queries
#[derive(Debug, Default, Deserialize)]
pub struct PromptFilter {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub workspace_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub status: Option<PromptStatus>,
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "type")]
    pub type_: Option<PromptType>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new prompt in the database
    #[instrument(skip(self))]
    pub async fn create_prompt(&self, prompt: &Prompt) -> Result<()> {
        debug!("Creating prompt with ID: {}", prompt.id);

        let _result = sqlx::query(
            "INSERT INTO prompts (
                    id, name, description, workspace_id, type, status, metadata, template, variables, tags, category, source, source_url, is_public, is_system, is_archived, is_deleted, is_featured, is_verified, created_by, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                )"
        )
        .bind(prompt.id)
        .bind(&prompt.name)
        .bind(&prompt.description)
        .bind(prompt.workspace_id)
        .bind(prompt.type_ as i32)
        .bind(prompt.status as i32)
        .bind(&prompt.metadata)
        .bind(&prompt.template)
        .bind(&prompt.variables)
        .bind(&prompt.tags)
        .bind(&prompt.category)
        .bind(&prompt.source)
        .bind(&prompt.source_url)
        .bind(prompt.is_public)
        .bind(prompt.is_system)
        .bind(prompt.is_archived)
        .bind(prompt.is_deleted)
        .bind(prompt.is_featured)
        .bind(prompt.is_verified)
        .bind(prompt.created_by)
        .bind(prompt.created_at)
        .bind(prompt.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_prompt_by_id(&self, id: &Uuid) -> Result<Option<Prompt>> {
        debug!("Getting prompt by ID: {}", id);

        let row = sqlx::query(
            r#"SELECT 
                    id, name, description, workspace_id, type, status, metadata, template, variables, tags, category, source, source_url, is_public, is_system, is_archived, is_deleted, is_featured, is_verified, created_by, created_at, updated_at
                FROM prompts WHERE id = ?"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let prompt = Prompt {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                type_: PromptType::try_from(row.get::<i64, _>("type") as i32)?,
                status: PromptStatus::try_from(row.get::<i64, _>("status") as i32)?,
                metadata: row.get("metadata"),
                template: row.get("template"),
                variables: row.get("variables"),
                tags: row.get("tags"),
                category: row.get("category"),
                source: row.get("source"),
                source_url: row.get("source_url"),
                is_public: row.get::<i64, _>("is_public") != 0,
                is_system: row.get::<i64, _>("is_system") != 0,
                is_archived: row.get::<i64, _>("is_archived") != 0,
                is_deleted: row.get::<i64, _>("is_deleted") != 0,
                is_featured: row.get::<i64, _>("is_featured") != 0,
                is_verified: row.get::<i64, _>("is_verified") != 0,
                created_by: row
                    .get::<Vec<u8>, _>("created_by")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(prompt))
        } else {
            Ok(None)
        }
    }

    /// List and filter prompts
    #[instrument(err, skip(self, filter))]
    pub async fn list_prompts(&self, filter: &PromptFilter) -> Result<Vec<Prompt>> {
        debug!("Listing prompts with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, name, description, workspace_id, type, status, metadata, template, variables, tags, category, source, source_url, is_public, is_system, is_archived, is_deleted, is_featured, is_verified, created_by, created_at, updated_at FROM prompts"#,
        );

        let mut where_conditions: Vec<String> = Vec::new();

        if let Some(workspace_id) = &filter.workspace_id {
            where_conditions.push(format!("workspace_id = '{workspace_id}'"));
        }

        if let Some(status) = filter.status {
            where_conditions.push(format!("status = {}", status as i64));
        }

        if let Some(type_) = filter.type_ {
            where_conditions.push(format!("type = {}", type_ as i64));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(name LIKE '%{search_term}%' OR description LIKE '%{search_term}%')"
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

        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut prompts = Vec::new();
        for row in rows {
            let prompt = Prompt {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                type_: PromptType::try_from(row.get::<i64, _>("type") as i32)?,
                status: PromptStatus::try_from(row.get::<i64, _>("status") as i32)?,
                metadata: row.get("metadata"),
                template: row.get("template"),
                variables: row.get("variables"),
                tags: row.get("tags"),
                category: row.get("category"),
                source: row.get("source"),
                source_url: row.get("source_url"),
                is_public: row.get::<i64, _>("is_public") != 0,
                is_system: row.get::<i64, _>("is_system") != 0,
                is_archived: row.get::<i64, _>("is_archived") != 0,
                is_deleted: row.get::<i64, _>("is_deleted") != 0,
                is_featured: row.get::<i64, _>("is_featured") != 0,
                is_verified: row.get::<i64, _>("is_verified") != 0,
                created_by: row
                    .get::<Vec<u8>, _>("created_by")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            prompts.push(prompt);
        }

        Ok(prompts)
    }

    /// Update a prompt
    #[instrument(err, skip(self))]
    pub async fn update_prompt(&self, prompt: &Prompt) -> Result<()> {
        debug!("Updating prompt with ID: {}", prompt.id);

        let affected = sqlx::query(
            "UPDATE prompts SET 
                name = ?, description = ?, workspace_id = ?, type = ?, status = ?, metadata = ?, template = ?, variables = ?, tags = ?, category = ?, source = ?, source_url = ?, is_public = ?, is_system = ?, is_archived = ?, is_deleted = ?, is_featured = ?, is_verified = ?, updated_at = ?
            WHERE id = ?"
        )
        .bind(&prompt.name)
        .bind(&prompt.description)
        .bind(prompt.workspace_id)
        .bind(prompt.type_ as i32)
        .bind(prompt.status as i32)
        .bind(&prompt.metadata)
        .bind(&prompt.template)
        .bind(&prompt.variables)
        .bind(&prompt.tags)
        .bind(&prompt.category)
        .bind(&prompt.source)
        .bind(&prompt.source_url)
        .bind(prompt.is_public)
        .bind(prompt.is_system)
        .bind(prompt.is_archived)
        .bind(prompt.is_deleted)
        .bind(prompt.is_featured)
        .bind(prompt.is_verified)
        .bind(prompt.updated_at)
        .bind(prompt.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Prompt with ID {} not found for update",
                prompt.id
            )));
        }

        Ok(())
    }

    /// Delete a prompt by ID
    #[instrument(err, skip(self))]
    pub async fn delete_prompt(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting prompt with ID: {}", id);

        let affected = sqlx::query("DELETE FROM prompts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Prompt with ID {id} not found for delete"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::DatabaseManager;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_and_get_prompt() {
        let db = DatabaseManager::setup_test_db().await;
        let prompt_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let workspace_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let user_id = Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap();

        let prompt = Prompt {
            id: prompt_id,
            name: "Test Prompt".to_string(),
            description: Some("Test Description".to_string()),
            workspace_id,
            type_: PromptType::System,
            status: PromptStatus::Active,
            metadata: r#"{"key": "value"}"#.to_string(),
            template: r#"{"name": "Test Prompt", "content": "Test template content"}"#.to_string(),
            variables: r#"{"variables": []}"#.to_string(),
            tags: r#"{"tags": []}"#.to_string(),
            category: r#"System"#.to_string(),
            source: None,
            source_url: None,
            is_public: false,
            is_system: true,
            is_archived: false,
            is_deleted: false,
            is_featured: false,
            is_verified: false,
            created_by: user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the prompt
        db.create_prompt(&prompt)
            .await
            .expect("Failed to create prompt");

        // Get the prompt
        let retrieved = db
            .get_prompt_by_id(&prompt_id)
            .await
            .expect("Failed to get prompt");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, prompt_id);
        assert_eq!(retrieved.name, "Test Prompt");
        assert_eq!(retrieved.description, Some("Test Description".to_string()));
        assert_eq!(retrieved.workspace_id, workspace_id);
        assert_eq!(retrieved.type_, PromptType::System);
        assert_eq!(retrieved.status, PromptStatus::Active);
        assert_eq!(retrieved.is_system, true);
        assert_eq!(retrieved.created_by, user_id);
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let db = DatabaseManager::setup_test_db().await;
        let workspace_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let user_id = Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap();

        // Create multiple prompts
        for i in 1..=3 {
            let prompt_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i + 1)).unwrap();
            let prompt = Prompt {
                id: prompt_id,
                name: format!("Prompt {}", i),
                description: Some(format!("Description {}", i)),
                workspace_id,
                type_: if i % 2 == 0 {
                    PromptType::System
                } else {
                    PromptType::Agent
                },
                status: PromptStatus::Active,
                metadata: r#"{"key": "value"}"#.to_string(),
                template: format!(
                    r#"{{"name": "Template {}", "content": "Template content {}"}}"#,
                    i, i
                ),
                variables: r#"{"variables": []}"#.to_string(),
                tags: r#"{"tags": []}"#.to_string(),
                category: r#"System"#.to_string(),
                source: None,
                source_url: None,
                is_public: false,
                is_system: false,
                is_archived: false,
                is_deleted: false,
                is_featured: false,
                is_verified: false,
                created_by: user_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            db.create_prompt(&prompt)
                .await
                .expect("Failed to create prompt");
        }

        // List all prompts
        let filter = PromptFilter::default();
        let prompts = db
            .list_prompts(&filter)
            .await
            .expect("Failed to list prompts");
        assert_eq!(prompts.len(), 3);

        // Filter by workspace_id
        let filter = PromptFilter {
            workspace_id: Some(workspace_id),
            ..Default::default()
        };
        let prompts = db
            .list_prompts(&filter)
            .await
            .expect("Failed to list prompts");
        assert_eq!(prompts.len(), 3);

        // Filter by type
        let filter = PromptFilter {
            type_: Some(PromptType::System),
            ..Default::default()
        };
        let prompts = db
            .list_prompts(&filter)
            .await
            .expect("Failed to list prompts");
        assert_eq!(prompts.len(), 1);

        // Filter by search term
        let filter = PromptFilter {
            search_term: Some("Prompt 2".to_string()),
            ..Default::default()
        };
        let prompts = db
            .list_prompts(&filter)
            .await
            .expect("Failed to list prompts");
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "Prompt 2");
    }

    #[tokio::test]
    async fn test_update_prompt() {
        let db = DatabaseManager::setup_test_db().await;
        let prompt_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let workspace_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let user_id = Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap();

        let prompt = Prompt {
            id: prompt_id,
            name: "Test Prompt".to_string(),
            description: Some("Test Description".to_string()),
            workspace_id,
            type_: PromptType::System,
            status: PromptStatus::Active,
            metadata: r#"{"key": "value"}"#.to_string(),
            template: r#"{"name": "Test Prompt", "content": "Test template content"}"#.to_string(),
            variables: r#"{"variables": []}"#.to_string(),
            tags: r#"{"tags": []}"#.to_string(),
            category: r#"System"#.to_string(),
            source: None,
            source_url: None,
            is_public: false,
            is_system: true,
            is_archived: false,
            is_deleted: false,
            is_featured: false,
            is_verified: false,
            created_by: user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the prompt
        db.create_prompt(&prompt)
            .await
            .expect("Failed to create prompt");

        // Update the prompt
        let updated_prompt = Prompt {
            id: prompt_id,
            name: "Updated Prompt".to_string(),
            description: Some("Updated Description".to_string()),
            workspace_id,
            type_: PromptType::System,
            status: PromptStatus::Archived,
            metadata: r#"{"updated": true}"#.to_string(),
            template: r#"{"name": "Updated Prompt", "content": "Updated template content"}"#
                .to_string(),
            variables: r#"{"variables": []}"#.to_string(),
            tags: r#"{"tags": []}"#.to_string(),
            category: r#"System"#.to_string(),
            source: None,
            source_url: None,
            is_public: false,
            is_system: true,
            is_archived: true,
            is_deleted: false,
            is_featured: false,
            is_verified: false,
            created_by: user_id,
            created_at: prompt.created_at,
            updated_at: Utc::now(),
        };

        db.update_prompt(&updated_prompt)
            .await
            .expect("Failed to update prompt");

        // Get the updated prompt
        let retrieved = db
            .get_prompt_by_id(&prompt_id)
            .await
            .expect("Failed to get prompt")
            .unwrap();
        assert_eq!(retrieved.name, "Updated Prompt");
        assert_eq!(
            retrieved.description,
            Some("Updated Description".to_string())
        );
        assert_eq!(retrieved.status, PromptStatus::Archived);
        assert_eq!(retrieved.is_archived, true);
    }

    #[tokio::test]
    async fn test_delete_prompt() {
        let db = DatabaseManager::setup_test_db().await;
        let prompt_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let workspace_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let user_id = Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap();

        let prompt = Prompt {
            id: prompt_id,
            name: "Test Prompt".to_string(),
            description: Some("Test Description".to_string()),
            workspace_id,
            type_: PromptType::System,
            status: PromptStatus::Active,
            metadata: r#"{"key": "value"}"#.to_string(),
            template: r#"{"name": "Test Prompt", "content": "Test template content"}"#.to_string(),
            variables: r#"{"variables": []}"#.to_string(),
            tags: r#"{"tags": []}"#.to_string(),
            category: r#"System"#.to_string(),
            source: None,
            source_url: None,
            is_public: false,
            is_system: true,
            is_archived: false,
            is_deleted: false,
            is_featured: false,
            is_verified: false,
            created_by: user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the prompt
        db.create_prompt(&prompt)
            .await
            .expect("Failed to create prompt");

        // Delete the prompt
        db.delete_prompt(&prompt_id)
            .await
            .expect("Failed to delete prompt");

        // Try to get the deleted prompt
        let retrieved = db
            .get_prompt_by_id(&prompt_id)
            .await
            .expect("Failed to query prompt");
        assert!(retrieved.is_none());
    }
}
