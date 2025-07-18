use async_trait::async_trait;
use super::r#trait::StorageBackend;
use crate::error::AppError;
// Note: Agent entity will be implemented in the future
// use crate::entities::agents::Agent;
use sqlx::PgPool;

pub struct PostgresBackend {
    pool: PgPool,
}

impl PostgresBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StorageBackend for PostgresBackend {
    // These methods will be implemented when Agent entity is ready
    // async fn get_agent(&self, id: &str) -> Result<Option<Agent>, AppError> {
    //     // Implementation here
    //     todo!()
    // }

    // async fn save_agent(&self, agent: &Agent) -> Result<(), AppError> {
    //     // Implementation here
    //     todo!()
    // }
} 