//! Security Services
//!
//! This module provides security-related services for the application,
//! including threat modeling and security testing.

use std::sync::Arc;
use anyhow::Result;
use sqlx::SqlitePool;
use tracing::{debug, info, warn};

use crate::security::threat_modeling::{
    ThreatModelingService, ThreatModel, Threat, ThreatSeverity, ThreatCategory,
    Asset, AssetSensitivity, TrustBoundary, TrustBoundaryType, DataFlow,
};
use crate::security::secure_defaults::{
    SecureDefaultsService, SecureDefaultsConfig,
};

/// Service for security-related functionality
#[derive(Debug)]
pub struct SecurityService {
    /// Threat modeling service
    threat_modeling: Arc<ThreatModelingService>,

    /// Secure defaults service
    secure_defaults: Arc<SecureDefaultsService>,
}

impl SecurityService {
    /// Create a new SecurityService
    pub async fn new(db: SqlitePool) -> Result<Self> {
        // Initialize the threat modeling service
        let threat_modeling = Arc::new(ThreatModelingService::new(db.clone()).await?);

        // Initialize the secure defaults service
        let secure_defaults = Arc::new(SecureDefaultsService::new(db, None).await?);

        Ok(Self {
            threat_modeling,
            secure_defaults,
        })
    }

    /// Get the secure defaults service
    pub fn secure_defaults(&self) -> Arc<SecureDefaultsService> {
        self.secure_defaults.clone()
    }

    /// Apply secure defaults to all components
    pub async fn apply_secure_defaults(&self) -> Result<()> {
        info!("Applying secure defaults to all components");
        self.secure_defaults.apply_secure_defaults().await?;
        info!("Secure defaults applied successfully");
        Ok(())
    }

    /// Get the threat modeling service
    pub fn threat_modeling(&self) -> Arc<ThreatModelingService> {
        self.threat_modeling.clone()
    }

    /// Initialize default threat models for the application
    pub async fn initialize_default_models(&self) -> Result<()> {
        info!("Initializing default threat models");

        // Check if we already have models
        let existing_models = self.threat_modeling.get_all_models().await?;
        if !existing_models.is_empty() {
            debug!("Found {} existing threat models, skipping initialization", existing_models.len());
            return Ok(());
        }

        // Create a threat model for the application
        let model_id = self.threat_modeling.create_model(
            "Evo Design Application",
            "Threat model for the main application",
        ).await?;

        // Get the model
        let mut model = self.threat_modeling.get_model(model_id).await?;

        // Add assets
        let user_data = Asset::new(
            "User Data",
            "Personal information and preferences of users",
            AssetSensitivity::Confidential,
        );

        let authentication_credentials = Asset::new(
            "Authentication Credentials",
            "User credentials for authentication",
            AssetSensitivity::Critical,
        );

        let application_code = Asset::new(
            "Application Code",
            "Source code and compiled binaries of the application",
            AssetSensitivity::Restricted,
        );

        model.add_asset(user_data);
        model.add_asset(authentication_credentials);
        model.add_asset(application_code);

        // Add trust boundaries
        let client_boundary = TrustBoundary::new(
            "Client Boundary",
            "Boundary between the user and the client application",
            TrustBoundaryType::Process,
        );

        let network_boundary = TrustBoundary::new(
            "Network Boundary",
            "Boundary between the client and external services",
            TrustBoundaryType::Network,
        );

        model.add_trust_boundary(client_boundary);
        model.add_trust_boundary(network_boundary);

        // Add data flows
        let authentication_flow = DataFlow::new(
            "User Authentication",
            "Flow of authentication data between user and system",
            "User",
            "Authentication System",
            "Credentials",
        )
        .with_authentication("Password")
        .with_encryption(true);

        let data_sync_flow = DataFlow::new(
            "Data Synchronization",
            "Synchronization of user data across devices",
            "Local Storage",
            "Remote Storage",
            "User Data",
        )
        .with_protocol("HTTPS")
        .with_encryption(true);

        model.add_data_flow(authentication_flow);
        model.add_data_flow(data_sync_flow);

        // Update the model
        self.threat_modeling.update_model(model).await?;

        // Generate common threats
        let threat_ids = self.threat_modeling.generate_common_threats(model_id).await?;
        info!("Generated {} common threats for the default model", threat_ids.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory SQLite database");

        db
    }

    #[tokio::test]
    async fn test_service_initialization() -> Result<()> {
        let db = setup_test_db().await;
        let service = SecurityService::new(db).await?;

        // Verify that the threat modeling service is initialized
        assert!(service.threat_modeling().get_all_models().await.is_ok());

        // Verify that the secure defaults service is initialized
        assert!(service.secure_defaults().apply_secure_defaults().await.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_apply_secure_defaults() -> Result<()> {
        let db = setup_test_db().await;
        let service = SecurityService::new(db).await?;

        // Apply secure defaults
        service.apply_secure_defaults().await?;

        // Verify that secure defaults were applied
        // (This is a basic test - the actual verification is done in the SecureDefaultsService tests)

        Ok(())
    }

    #[tokio::test]
    async fn test_initialize_default_models() -> Result<()> {
        let db = setup_test_db().await;
        let service = SecurityService::new(db).await?;

        // Initialize default models
        service.initialize_default_models().await?;

        // Get all models
        let models = service.threat_modeling().get_all_models().await?;

        // Should have at least one model
        assert!(!models.is_empty());

        // The model should have assets, trust boundaries, data flows, and threats
        let model = &models[0];
        assert!(!model.assets.is_empty());
        assert!(!model.trust_boundaries.is_empty());
        assert!(!model.data_flows.is_empty());
        assert!(!model.threats.is_empty());

        Ok(())
    }
}
