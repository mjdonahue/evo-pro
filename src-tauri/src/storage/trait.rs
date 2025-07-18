use async_trait::async_trait;

use crate::error::AppError;

// Note: Agent entity will be implemented in the future
// use crate::entities::agents::Agent;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    // These methods will be implemented when Agent entity is ready
    // async fn get_agent(&self, id: &str) -> Result<Option<Agent>, AppError>;
    // async fn save_agent(&self, agent: &Agent) -> Result<(), AppError>;
    // Add more methods as needed for your use case
} 