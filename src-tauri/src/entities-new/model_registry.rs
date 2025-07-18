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
pub enum ModelType {
    Llm = 0,
    Mcp = 1,
    Tool = 2,
    Other = 3,
}

impl TryFrom<i32> for ModelType {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(ModelType::Llm),
            1 => Ok(ModelType::Mcp),
            2 => Ok(ModelType::Tool),
            3 => Ok(ModelType::Other),
            _ => Err(AppError::DeserializationError(format!(
                "Invalid ModelType: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ModelRegistryStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}

impl TryFrom<i32> for ModelRegistryStatus {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(ModelRegistryStatus::Active),
            1 => Ok(ModelRegistryStatus::Archived),
            2 => Ok(ModelRegistryStatus::Deleted),
            _ => Err(AppError::DeserializationError(format!(
                "Invalid ModelRegistryStatus: {value}"
            ))),
        }
    }
}

/// ModelRegistry model matching the SQLite schema
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ModelRegistry {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    pub provider: String,
    pub model_type: ModelType,
    pub version: String,
    pub context_size: i64,
    pub max_tokens: i64,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub input_cost_per_1k: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub output_cost_per_1k: Option<f64>,
    pub config: String,       // JSON object with configuration
    pub capabilities: String, // JSON array of capabilities
    pub tags: String,         // JSON array of tags
    pub is_deprecated: bool,
    pub status: ModelRegistryStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for model registry queries
#[derive(Debug, Default, Deserialize)]
pub struct ModelRegistryFilter {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model_type: Option<ModelType>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub status: Option<ModelRegistryStatus>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub supports_functions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub supports_vision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub supports_streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub min_context_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub capability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new model registry entry in the database
    #[instrument(skip(self))]
    pub async fn create_model_registry(&self, model_registry: &ModelRegistry) -> Result<()> {
        debug!("Creating model registry with ID: {}", model_registry.id);

        let _result = sqlx::query(
            "INSERT INTO model_registry (
                    id, name, description, provider, model_type, version, context_size,
                    max_tokens, supports_functions, supports_vision, supports_streaming,
                    input_cost_per_1k, output_cost_per_1k, config, capabilities, tags,
                    is_deprecated, status, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                )",
        )
        .bind(model_registry.id)
        .bind(&model_registry.name)
        .bind(&model_registry.description)
        .bind(&model_registry.provider)
        .bind(model_registry.model_type as i32)
        .bind(&model_registry.version)
        .bind(model_registry.context_size)
        .bind(model_registry.max_tokens)
        .bind(model_registry.supports_functions)
        .bind(model_registry.supports_vision)
        .bind(model_registry.supports_streaming)
        .bind(model_registry.input_cost_per_1k)
        .bind(model_registry.output_cost_per_1k)
        .bind(&model_registry.config)
        .bind(&model_registry.capabilities)
        .bind(&model_registry.tags)
        .bind(model_registry.is_deprecated)
        .bind(model_registry.status as i32)
        .bind(model_registry.created_at)
        .bind(model_registry.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_model_registry_by_id(&self, id: &Uuid) -> Result<Option<ModelRegistry>> {
        debug!("Getting model registry by ID: {}", id);

        let row = sqlx::query(
            r#"SELECT 
                    id, name, description, provider, model_type, version, context_size, max_tokens, supports_functions, supports_vision, supports_streaming, input_cost_per_1k, output_cost_per_1k, config, capabilities, tags, is_deprecated, status, created_at, updated_at
                FROM model_registry WHERE id = ?"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let model_registry = ModelRegistry {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                provider: row.get("provider"),
                model_type: ModelType::try_from(row.get::<i64, _>("model_type") as i32)?,
                version: row.get("version"),
                context_size: row.get("context_size"),
                max_tokens: row.get("max_tokens"),
                supports_functions: row.get::<i64, _>("supports_functions") != 0,
                supports_vision: row.get::<i64, _>("supports_vision") != 0,
                supports_streaming: row.get::<i64, _>("supports_streaming") != 0,
                input_cost_per_1k: row.get("input_cost_per_1k"),
                output_cost_per_1k: row.get("output_cost_per_1k"),
                config: row.get("config"),
                capabilities: row.get("capabilities"),
                tags: row.get("tags"),
                is_deprecated: row.get::<i64, _>("is_deprecated") != 0,
                status: ModelRegistryStatus::try_from(row.get::<i64, _>("status") as i32)?,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(model_registry))
        } else {
            Ok(None)
        }
    }

    /// List and filter model registry entries
    #[instrument(err, skip(self, filter))]
    pub async fn list_model_registry(
        &self,
        filter: &ModelRegistryFilter,
    ) -> Result<Vec<ModelRegistry>> {
        debug!("Listing model registry with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, name, description, provider, model_type,
               version, context_size, max_tokens, supports_functions, supports_vision,
               supports_streaming, input_cost_per_1k, output_cost_per_1k, config,
               capabilities, tags, is_deprecated, status,
               created_at, updated_at FROM model_registry"#,
        );

        let mut where_conditions: Vec<String> = Vec::new();

        if let Some(provider) = &filter.provider {
            where_conditions.push(format!("provider = '{provider}'"));
        }

        if let Some(model_type) = filter.model_type {
            where_conditions.push(format!("model_type = {}", model_type as i64));
        }

        if let Some(status) = filter.status {
            where_conditions.push(format!("status = {}", status as i64));
        }

        if let Some(is_deprecated) = filter.is_deprecated {
            where_conditions.push(format!(
                "is_deprecated = {}",
                if is_deprecated { 1 } else { 0 }
            ));
        }

        if let Some(supports_functions) = filter.supports_functions {
            where_conditions.push(format!(
                "supports_functions = {}",
                if supports_functions { 1 } else { 0 }
            ));
        }

        if let Some(supports_vision) = filter.supports_vision {
            where_conditions.push(format!(
                "supports_vision = {}",
                if supports_vision { 1 } else { 0 }
            ));
        }

        if let Some(supports_streaming) = filter.supports_streaming {
            where_conditions.push(format!(
                "supports_streaming = {}",
                if supports_streaming { 1 } else { 0 }
            ));
        }

        if let Some(min_context_size) = filter.min_context_size {
            where_conditions.push(format!("context_size >= {min_context_size}"));
        }

        if let Some(capability) = &filter.capability {
            where_conditions.push(format!("capabilities LIKE '%\"{capability}\"%'"));
        }

        if let Some(tag) = &filter.tag {
            where_conditions.push(format!("tags LIKE '%\"{tag}\"%'"));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(name LIKE '%{search_term}%' OR description LIKE '%{search_term}%' OR provider LIKE '%{search_term}%')"
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

        let mut model_registry_entries = Vec::new();
        for row in rows {
            let model_registry = ModelRegistry {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                provider: row.get("provider"),
                model_type: ModelType::try_from(row.get::<i64, _>("model_type") as i32)?,
                version: row.get("version"),
                context_size: row.get("context_size"),
                max_tokens: row.get("max_tokens"),
                supports_functions: row.get::<i64, _>("supports_functions") != 0,
                supports_vision: row.get::<i64, _>("supports_vision") != 0,
                supports_streaming: row.get::<i64, _>("supports_streaming") != 0,
                input_cost_per_1k: row.get("input_cost_per_1k"),
                output_cost_per_1k: row.get("output_cost_per_1k"),
                config: row.get("config"),
                capabilities: row.get("capabilities"),
                tags: row.get("tags"),
                is_deprecated: row.get::<i64, _>("is_deprecated") != 0,
                status: ModelRegistryStatus::try_from(row.get::<i64, _>("status") as i32)?,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            model_registry_entries.push(model_registry);
        }

        Ok(model_registry_entries)
    }

    /// Update a model registry entry
    #[instrument(err, skip(self))]
    pub async fn update_model_registry(&self, model_registry: &ModelRegistry) -> Result<()> {
        debug!("Updating model registry with ID: {}", model_registry.id);

        let affected = sqlx::query(
            "UPDATE model_registry SET 
                name = ?, description = ?, provider = ?, model_type = ?, version = ?,
                context_size = ?, max_tokens = ?, supports_functions = ?, supports_vision = ?,
                supports_streaming = ?, input_cost_per_1k = ?, output_cost_per_1k = ?,
                config = ?, capabilities = ?, tags = ?, is_deprecated = ?, status = ?,
                updated_at = ?
            WHERE id = ?",
        )
        .bind(&model_registry.name)
        .bind(&model_registry.description)
        .bind(&model_registry.provider)
        .bind(model_registry.model_type as i32)
        .bind(&model_registry.version)
        .bind(model_registry.context_size)
        .bind(model_registry.max_tokens)
        .bind(model_registry.supports_functions)
        .bind(model_registry.supports_vision)
        .bind(model_registry.supports_streaming)
        .bind(model_registry.input_cost_per_1k)
        .bind(model_registry.output_cost_per_1k)
        .bind(&model_registry.config)
        .bind(&model_registry.capabilities)
        .bind(&model_registry.tags)
        .bind(model_registry.is_deprecated)
        .bind(model_registry.status as i32)
        .bind(model_registry.updated_at)
        .bind(model_registry.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Model registry with ID {} not found for update",
                model_registry.id
            )));
        }

        Ok(())
    }

    /// Delete a model registry entry by ID
    #[instrument(err, skip(self))]
    pub async fn delete_model_registry(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting model registry with ID: {}", id);

        let affected = sqlx::query("DELETE FROM model_registry WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Model registry with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Mark a model as deprecated
    #[instrument(err, skip(self))]
    pub async fn deprecate_model(&self, id: &Uuid) -> Result<()> {
        debug!("Marking model with ID {} as deprecated", id);

        let affected =
            sqlx::query("UPDATE model_registry SET is_deprecated = 1, updated_at = ? WHERE id = ?")
                .bind(Utc::now())
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Model registry with ID {id} not found for deprecation"
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
    async fn test_create_and_get_model_registry() {
        let db = DatabaseManager::setup_test_db().await;
        let model_registry_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let model_registry = ModelRegistry {
            id: model_registry_id,
            name: "GPT-4".to_string(),
            description: Some("Advanced language model".to_string()),
            provider: "openai".to_string(),
            model_type: ModelType::Llm,
            version: "1.0.0".to_string(),
            context_size: 8192,
            max_tokens: 4096,
            supports_functions: true,
            supports_vision: true,
            supports_streaming: true,
            input_cost_per_1k: Some(0.01),
            output_cost_per_1k: Some(0.02),
            config: r#"{"temperature": 0.7}"#.to_string(),
            capabilities: r#"["text-generation", "code-generation"]"#.to_string(),
            tags: r#"["gpt", "large"]"#.to_string(),
            is_deprecated: false,
            status: ModelRegistryStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the model registry
        db.create_model_registry(&model_registry)
            .await
            .expect("Failed to create model registry");

        // Get the model registry
        let retrieved = db
            .get_model_registry_by_id(&model_registry_id)
            .await
            .expect("Failed to get model registry");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, model_registry_id);
        assert_eq!(retrieved.name, "GPT-4");
        assert_eq!(
            retrieved.description,
            Some("Advanced language model".to_string())
        );
        assert_eq!(retrieved.provider, "openai");
        assert_eq!(retrieved.model_type, ModelType::Llm);
        assert_eq!(retrieved.version, "1.0.0");
        assert_eq!(retrieved.context_size, 8192);
        assert_eq!(retrieved.max_tokens, 4096);
        assert_eq!(retrieved.supports_functions, true);
        assert_eq!(retrieved.supports_vision, true);
        assert_eq!(retrieved.supports_streaming, true);
        assert_eq!(retrieved.input_cost_per_1k, Some(0.01));
        assert_eq!(retrieved.output_cost_per_1k, Some(0.02));
        assert_eq!(retrieved.config, r#"{"temperature": 0.7}"#);
        assert_eq!(
            retrieved.capabilities,
            r#"["text-generation", "code-generation"]"#
        );
        assert_eq!(retrieved.tags, r#"["gpt", "large"]"#);
        assert_eq!(retrieved.is_deprecated, false);
        assert_eq!(retrieved.status, ModelRegistryStatus::Active);
    }

    #[tokio::test]
    async fn test_list_model_registry() {
        let db = DatabaseManager::setup_test_db().await;

        // Create multiple model registry entries
        for i in 1..=3 {
            let model_registry_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i + 1)).unwrap();
            let model_registry = ModelRegistry {
                id: model_registry_id,
                name: format!("Model {}", i),
                description: Some(format!("Description {}", i)),
                provider: if i % 2 == 0 {
                    "openai".to_string()
                } else {
                    "anthropic".to_string()
                },
                model_type: ModelType::Llm,
                version: format!("1.0.{}", i),
                context_size: 4096 * i as i64,
                max_tokens: 2048 * i as i64,
                supports_functions: i % 2 == 0,
                supports_vision: i == 3,
                supports_streaming: true,
                input_cost_per_1k: Some(0.01 * i as f64),
                output_cost_per_1k: Some(0.02 * i as f64),
                config: r#"{"temperature": 0.7}"#.to_string(),
                capabilities: if i % 2 == 0 {
                    r#"["text-generation", "code-generation"]"#.to_string()
                } else {
                    r#"["text-generation"]"#.to_string()
                },
                tags: if i % 2 == 0 {
                    r#"["gpt", "large"]"#.to_string()
                } else {
                    r#"["claude", "medium"]"#.to_string()
                },
                is_deprecated: i == 1,
                status: ModelRegistryStatus::Active,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            db.create_model_registry(&model_registry)
                .await
                .expect("Failed to create model registry");
        }

        // List all model registry entries
        let filter = ModelRegistryFilter::default();
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 3);

        // Filter by provider
        let filter = ModelRegistryFilter {
            provider: Some("openai".to_string()),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Model 2");

        // Filter by supports_functions
        let filter = ModelRegistryFilter {
            supports_functions: Some(true),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Model 2");

        // Filter by supports_vision
        let filter = ModelRegistryFilter {
            supports_vision: Some(true),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Model 3");

        // Filter by is_deprecated
        let filter = ModelRegistryFilter {
            is_deprecated: Some(true),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Model 1");

        // Filter by min_context_size
        let filter = ModelRegistryFilter {
            min_context_size: Some(8192),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 2);

        // Filter by capability
        let filter = ModelRegistryFilter {
            capability: Some("code-generation".to_string()),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Model 2");

        // Filter by tag
        let filter = ModelRegistryFilter {
            tag: Some("claude".to_string()),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "Model 1"));
        assert!(entries.iter().any(|e| e.name == "Model 3"));

        // Filter by search term
        let filter = ModelRegistryFilter {
            search_term: Some("Model 2".to_string()),
            ..Default::default()
        };
        let entries = db
            .list_model_registry(&filter)
            .await
            .expect("Failed to list model registry");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Model 2");
    }

    #[tokio::test]
    async fn test_update_model_registry() {
        let db = DatabaseManager::setup_test_db().await;
        let model_registry_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let model_registry = ModelRegistry {
            id: model_registry_id,
            name: "GPT-4".to_string(),
            description: Some("Advanced language model".to_string()),
            provider: "openai".to_string(),
            model_type: ModelType::Llm,
            version: "1.0.0".to_string(),
            context_size: 8192,
            max_tokens: 4096,
            supports_functions: true,
            supports_vision: false,
            supports_streaming: true,
            input_cost_per_1k: Some(0.01),
            output_cost_per_1k: Some(0.02),
            config: r#"{"temperature": 0.7}"#.to_string(),
            capabilities: r#"["text-generation"]"#.to_string(),
            tags: r#"["gpt", "large"]"#.to_string(),
            is_deprecated: false,
            status: ModelRegistryStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the model registry
        db.create_model_registry(&model_registry)
            .await
            .expect("Failed to create model registry");

        // Update the model registry
        let updated_model_registry = ModelRegistry {
            id: model_registry_id,
            name: "GPT-4 Turbo".to_string(),
            description: Some("Enhanced language model".to_string()),
            provider: "openai".to_string(),
            model_type: ModelType::Llm,
            version: "1.1.0".to_string(),
            context_size: 16384,
            max_tokens: 8192,
            supports_functions: true,
            supports_vision: true,
            supports_streaming: true,
            input_cost_per_1k: Some(0.015),
            output_cost_per_1k: Some(0.025),
            config: r#"{"temperature": 0.8, "top_p": 0.95}"#.to_string(),
            capabilities: r#"["text-generation", "code-generation", "vision"]"#.to_string(),
            tags: r#"["gpt", "large", "turbo"]"#.to_string(),
            is_deprecated: false,
            status: ModelRegistryStatus::Active,
            created_at: model_registry.created_at,
            updated_at: Utc::now(),
        };

        db.update_model_registry(&updated_model_registry)
            .await
            .expect("Failed to update model registry");

        // Get the updated model registry
        let retrieved = db
            .get_model_registry_by_id(&model_registry_id)
            .await
            .expect("Failed to get model registry")
            .unwrap();
        assert_eq!(retrieved.name, "GPT-4 Turbo");
        assert_eq!(
            retrieved.description,
            Some("Enhanced language model".to_string())
        );
        assert_eq!(retrieved.version, "1.1.0");
        assert_eq!(retrieved.context_size, 16384);
        assert_eq!(retrieved.max_tokens, 8192);
        assert_eq!(retrieved.supports_vision, true);
        assert_eq!(retrieved.input_cost_per_1k, Some(0.015));
        assert_eq!(retrieved.output_cost_per_1k, Some(0.025));
        assert_eq!(retrieved.config, r#"{"temperature": 0.8, "top_p": 0.95}"#);
        assert_eq!(
            retrieved.capabilities,
            r#"["text-generation", "code-generation", "vision"]"#
        );
        assert_eq!(retrieved.tags, r#"["gpt", "large", "turbo"]"#);
    }

    #[tokio::test]
    async fn test_delete_model_registry() {
        let db = DatabaseManager::setup_test_db().await;
        let model_registry_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let model_registry = ModelRegistry {
            id: model_registry_id,
            name: "GPT-4".to_string(),
            description: None,
            provider: "openai".to_string(),
            model_type: ModelType::Llm,
            version: "1.0.0".to_string(),
            context_size: 8192,
            max_tokens: 4096,
            supports_functions: true,
            supports_vision: false,
            supports_streaming: true,
            input_cost_per_1k: None,
            output_cost_per_1k: None,
            config: r#"{}"#.to_string(),
            capabilities: r#"[]"#.to_string(),
            tags: r#"[]"#.to_string(),
            is_deprecated: false,
            status: ModelRegistryStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the model registry
        db.create_model_registry(&model_registry)
            .await
            .expect("Failed to create model registry");

        // Delete the model registry
        db.delete_model_registry(&model_registry_id)
            .await
            .expect("Failed to delete model registry");

        // Try to get the deleted model registry
        let retrieved = db
            .get_model_registry_by_id(&model_registry_id)
            .await
            .expect("Failed to query model registry");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_deprecate_model() {
        let db = DatabaseManager::setup_test_db().await;
        let model_registry_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let model_registry = ModelRegistry {
            id: model_registry_id,
            name: "GPT-4".to_string(),
            description: None,
            provider: "openai".to_string(),
            model_type: ModelType::Llm,
            version: "1.0.0".to_string(),
            context_size: 8192,
            max_tokens: 4096,
            supports_functions: true,
            supports_vision: false,
            supports_streaming: true,
            input_cost_per_1k: None,
            output_cost_per_1k: None,
            config: r#"{}"#.to_string(),
            capabilities: r#"[]"#.to_string(),
            tags: r#"[]"#.to_string(),
            is_deprecated: false,
            status: ModelRegistryStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the model registry
        db.create_model_registry(&model_registry)
            .await
            .expect("Failed to create model registry");

        // Deprecate the model
        db.deprecate_model(&model_registry_id)
            .await
            .expect("Failed to deprecate model");

        // Get the updated model registry
        let retrieved = db
            .get_model_registry_by_id(&model_registry_id)
            .await
            .expect("Failed to get model registry")
            .unwrap();
        assert_eq!(retrieved.is_deprecated, true);
    }
}
