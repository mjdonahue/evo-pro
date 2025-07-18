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
pub enum ModelType {
    Llm = 0,
    Mcp = 1,
    Tool = 2,
    Other = 3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ModelProvider {
    OpenAI = 0,
    Cohere = 1,
    Anthropic = 2,
    Perplexity = 3,
    Gemini = 4,
    XAi = 5,
    DeepSeek = 6,
    Ollama = 7,
}

#[boilermates("CreateModel")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[boilermates(not_in("CreateModel"))]
    pub id: Uuid,
    pub provider: ModelProvider,
    pub name: String,
    pub display_name: Option<String>,
    pub model_type: ModelType,
    pub context_size: i64,
    pub max_tokens: i64,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    pub input_cost: Option<f64>,
    pub output_cost: Option<f64>,
    pub config: Option<Json<Value>>, // JSON object with configuration
    pub is_active: bool,
    pub is_deprecated: bool,
    pub registry_id: Option<Uuid>,
    #[boilermates(not_in("CreateModel"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateModel"))]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for model queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct ModelFilter {
    pub name: Option<String>,
    pub model_type: Option<ModelType>,
    pub provider: Option<ModelProvider>,
    pub search_term: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new model in the database
    #[instrument(skip(self))]
    pub async fn create_model(&self, model: &CreateModel) -> Result<Model> {
        let id = Uuid::new_v4();
        debug!("Creating model with ID: {}", id);
        let config = model.config.as_deref();
        let now = Utc::now();

        Ok(sqlx::query_as!(
            Model,
            r#"INSERT INTO models (
                    id, provider, name, display_name,
                    model_type, context_size, max_tokens, supports_functions,
                    supports_vision, supports_streaming, input_cost, output_cost,
                    config, is_active, is_deprecated, registry_id, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING
                    id AS "id: _", provider AS "provider: _",
                    name, display_name,
                    model_type as "model_type: ModelType",
                    context_size, max_tokens, supports_functions,
                    supports_vision, supports_streaming, input_cost, output_cost,
                    config AS "config: _", is_active, is_deprecated,
                    registry_id AS "registry_id: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            id,
            model.provider,
            model.name,
            model.display_name,
            model.model_type,
            model.context_size,
            model.max_tokens,
            model.supports_functions,
            model.supports_vision,
            model.supports_streaming,
            model.input_cost,
            model.output_cost,
            config,
            model.is_active,
            model.is_deprecated,
            model.registry_id,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?)
    }

    #[instrument(err, skip(self))]
    pub async fn get_model_by_id(&self, id: &Uuid) -> Result<Option<Model>> {
        debug!("Getting model by ID: {}", id);

        Ok(sqlx::query_as!(
            Model,
            r#"SELECT
                    id AS "id: _", provider AS "provider: _",
                    name, display_name, model_type as "model_type: ModelType", context_size, max_tokens, supports_functions,
                    supports_vision, supports_streaming, input_cost, output_cost,
                    config AS "config: _", is_active, is_deprecated,
                    registry_id AS "registry_id: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM models WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get model by provider and name
    #[instrument(skip(self))]
    pub async fn get_model_by_provider_and_name(
        &self,
        provider: &ModelProvider,
        name: &str,
    ) -> Result<Option<Model>> {
        debug!(
            "Getting model for provider {:?} and name {}",
            provider, name
        );

        Ok(sqlx::query_as!(
            Model,
            r#"SELECT
                    id AS "id: _", provider AS "provider: _",
                    name, display_name, model_type as "model_type: ModelType", context_size, max_tokens, supports_functions,
                    supports_vision, supports_streaming, input_cost, output_cost,
                    config AS "config: _", is_active, is_deprecated,
                    registry_id AS "registry_id: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM models
                WHERE provider = ? AND name = ?"#,
            provider,
            name
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter models
    #[instrument(err, skip(self, filter))]
    pub async fn list_models(&self, filter: &ModelFilter) -> Result<Vec<Model>> {
        debug!("Listing models with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, provider, name, display_name,
               model_type, context_size, max_tokens, supports_functions, supports_vision, supports_streaming,
               input_cost, output_cost, config, is_active, is_deprecated,
               registry_id, created_at, updated_at
               FROM models"#,
        );

        let mut add_where = add_where();

        if let Some(name) = &filter.name {
            add_where(&mut qb);
            let pattern = format!("%{name}%");
            qb.push("(name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(")");
        }

        if let Some(model_type) = filter.model_type {
            add_where(&mut qb);
            qb.push("model_type = ");
            qb.push_bind(model_type);
        }

        if let Some(provider) = &filter.provider {
            add_where(&mut qb);
            qb.push("provider = ");
            qb.push_bind(provider);
        }

        if let Some(is_active) = &filter.is_active {
            add_where(&mut qb);
            qb.push("is_active = ");
            qb.push_bind(is_active);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            let pattern = format!("%{}%", search_term);
            qb.push("(name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR description LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        qb.push(" ORDER BY updated_at DESC");

        if let Some(limit) = filter.limit {
            add_where(&mut qb);
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            add_where(&mut qb);
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        Ok(qb
            .build_query_as::<'_, Model>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update a model
    #[instrument(err, skip(self))]
    pub async fn update_model(&self, model: &Model) -> Result<Model> {
        debug!("Updating model with ID: {:?}", model.id);

        Ok(sqlx::query_as!(
            Model,
            r#"UPDATE models SET
                name = ?, display_name = ?, provider = ?, registry_id = ?,
                model_type = ?, context_size = ?, max_tokens = ?, supports_functions = ?,
                supports_vision = ?, supports_streaming = ?, input_cost = ?, output_cost = ?,
                config = ?, is_active = ?, is_deprecated = ?, updated_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", provider AS "provider: _", registry_id AS "registry_id: _",
                name, display_name, model_type as "model_type: ModelType", context_size, max_tokens, supports_functions,
                supports_vision, supports_streaming, input_cost, output_cost,
                config AS "config: _", is_active, is_deprecated,
                created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            model.name,
            model.display_name,
            model.provider,
            model.registry_id,
            model.model_type,
            model.context_size,
            model.max_tokens,
            model.supports_functions,
            model.supports_vision,
            model.supports_streaming,
            model.input_cost,
            model.output_cost,
            model.config,
            model.is_deprecated,
            model.is_active,
            model.updated_at,
            model.id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Delete a model by ID
    #[instrument(err, skip(self))]
    pub async fn delete_model(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting model with ID: {}", id);

        let affected = sqlx::query!("DELETE FROM models WHERE id = ?", id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Model with ID {id} not found for delete"
            )));
        }

        Ok(())
    }
}
