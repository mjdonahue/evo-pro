//! Threat Modeling System
//!
//! This module provides a comprehensive threat modeling system for identifying,
//! assessing, and mitigating security threats to the application.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use sqlx::{SqlitePool, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Severity level of a security threat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatSeverity {
    /// Critical threats that require immediate attention
    Critical,
    /// High-severity threats that should be addressed soon
    High,
    /// Medium-severity threats that should be addressed in the normal course of development
    Medium,
    /// Low-severity threats that can be addressed when convenient
    Low,
    /// Informational items that don't necessarily require action
    Informational,
}

impl std::fmt::Display for ThreatSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Low => write!(f, "Low"),
            Self::Informational => write!(f, "Informational"),
        }
    }
}

/// Category of a security threat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatCategory {
    /// Threats related to authentication and authorization
    Authentication,
    /// Threats related to data validation and sanitization
    InputValidation,
    /// Threats related to sensitive data exposure
    DataExposure,
    /// Threats related to session management
    SessionManagement,
    /// Threats related to access control
    AccessControl,
    /// Threats related to cryptography
    Cryptography,
    /// Threats related to business logic
    BusinessLogic,
    /// Threats related to file operations
    FileOperations,
    /// Threats related to network communication
    NetworkCommunication,
    /// Threats related to third-party components
    ThirdPartyComponents,
    /// Threats related to configuration
    Configuration,
    /// Threats that don't fit into other categories
    Other,
}

impl std::fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authentication => write!(f, "Authentication"),
            Self::InputValidation => write!(f, "Input Validation"),
            Self::DataExposure => write!(f, "Data Exposure"),
            Self::SessionManagement => write!(f, "Session Management"),
            Self::AccessControl => write!(f, "Access Control"),
            Self::Cryptography => write!(f, "Cryptography"),
            Self::BusinessLogic => write!(f, "Business Logic"),
            Self::FileOperations => write!(f, "File Operations"),
            Self::NetworkCommunication => write!(f, "Network Communication"),
            Self::ThirdPartyComponents => write!(f, "Third-Party Components"),
            Self::Configuration => write!(f, "Configuration"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Status of a threat mitigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MitigationStatus {
    /// Mitigation has not been started
    NotStarted,
    /// Mitigation is in progress
    InProgress,
    /// Mitigation has been implemented but not verified
    Implemented,
    /// Mitigation has been verified
    Verified,
    /// Mitigation has been deferred
    Deferred,
    /// Mitigation has been accepted as a risk
    AcceptedRisk,
    /// Mitigation is not applicable
    NotApplicable,
}

impl std::fmt::Display for MitigationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "Not Started"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Implemented => write!(f, "Implemented"),
            Self::Verified => write!(f, "Verified"),
            Self::Deferred => write!(f, "Deferred"),
            Self::AcceptedRisk => write!(f, "Accepted Risk"),
            Self::NotApplicable => write!(f, "Not Applicable"),
        }
    }
}

/// Represents a security threat to the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    /// Unique identifier for the threat
    pub id: Uuid,
    
    /// Name of the threat
    pub name: String,
    
    /// Description of the threat
    pub description: String,
    
    /// Category of the threat
    pub category: ThreatCategory,
    
    /// Severity of the threat
    pub severity: ThreatSeverity,
    
    /// Affected components or assets
    pub affected_components: Vec<String>,
    
    /// Potential impact if the threat is realized
    pub potential_impact: String,
    
    /// Likelihood of the threat occurring (1-10)
    pub likelihood: u8,
    
    /// Mitigation strategies
    pub mitigations: Vec<Mitigation>,
    
    /// References to related resources (e.g., CWE, OWASP)
    pub references: HashMap<String, String>,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last update date
    pub updated_at: DateTime<Utc>,
}

impl Threat {
    /// Create a new threat
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: ThreatCategory,
        severity: ThreatSeverity,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            category,
            severity,
            affected_components: Vec::new(),
            potential_impact: String::new(),
            likelihood: 5, // Default to medium likelihood
            mitigations: Vec::new(),
            references: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add an affected component
    pub fn with_affected_component(mut self, component: impl Into<String>) -> Self {
        self.affected_components.push(component.into());
        self
    }
    
    /// Set the potential impact
    pub fn with_potential_impact(mut self, impact: impl Into<String>) -> Self {
        self.potential_impact = impact.into();
        self
    }
    
    /// Set the likelihood
    pub fn with_likelihood(mut self, likelihood: u8) -> Self {
        self.likelihood = likelihood.min(10); // Ensure likelihood is at most 10
        self
    }
    
    /// Add a mitigation strategy
    pub fn with_mitigation(mut self, mitigation: Mitigation) -> Self {
        self.mitigations.push(mitigation);
        self
    }
    
    /// Add a reference
    pub fn with_reference(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.references.insert(key.into(), value.into());
        self
    }
    
    /// Calculate the risk score (severity * likelihood)
    pub fn risk_score(&self) -> u8 {
        let severity_score = match self.severity {
            ThreatSeverity::Critical => 10,
            ThreatSeverity::High => 8,
            ThreatSeverity::Medium => 5,
            ThreatSeverity::Low => 2,
            ThreatSeverity::Informational => 1,
        };
        
        severity_score * self.likelihood / 10
    }
    
    /// Check if the threat is mitigated
    pub fn is_mitigated(&self) -> bool {
        !self.mitigations.is_empty() && self.mitigations.iter().any(|m| {
            m.status == MitigationStatus::Implemented || 
            m.status == MitigationStatus::Verified
        })
    }
    
    /// Update the threat's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Represents a mitigation strategy for a threat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mitigation {
    /// Unique identifier for the mitigation
    pub id: Uuid,
    
    /// Description of the mitigation
    pub description: String,
    
    /// Status of the mitigation
    pub status: MitigationStatus,
    
    /// Responsible party for implementing the mitigation
    pub responsible_party: Option<String>,
    
    /// Notes about the mitigation
    pub notes: Option<String>,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last update date
    pub updated_at: DateTime<Utc>,
}

impl Mitigation {
    /// Create a new mitigation
    pub fn new(description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            description: description.into(),
            status: MitigationStatus::NotStarted,
            responsible_party: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Set the status
    pub fn with_status(mut self, status: MitigationStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Set the responsible party
    pub fn with_responsible_party(mut self, party: impl Into<String>) -> Self {
        self.responsible_party = Some(party.into());
        self
    }
    
    /// Set notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
    
    /// Update the mitigation's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Represents a threat model for a component or system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatModel {
    /// Unique identifier for the threat model
    pub id: Uuid,
    
    /// Name of the component or system being modeled
    pub name: String,
    
    /// Description of the component or system
    pub description: String,
    
    /// Threats identified for this component or system
    pub threats: Vec<Threat>,
    
    /// Assets or resources being protected
    pub assets: Vec<Asset>,
    
    /// Trust boundaries
    pub trust_boundaries: Vec<TrustBoundary>,
    
    /// Data flows
    pub data_flows: Vec<DataFlow>,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last update date
    pub updated_at: DateTime<Utc>,
}

impl ThreatModel {
    /// Create a new threat model
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            threats: Vec::new(),
            assets: Vec::new(),
            trust_boundaries: Vec::new(),
            data_flows: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add a threat to the model
    pub fn add_threat(&mut self, threat: Threat) {
        self.threats.push(threat);
        self.touch();
    }
    
    /// Add an asset to the model
    pub fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
        self.touch();
    }
    
    /// Add a trust boundary to the model
    pub fn add_trust_boundary(&mut self, boundary: TrustBoundary) {
        self.trust_boundaries.push(boundary);
        self.touch();
    }
    
    /// Add a data flow to the model
    pub fn add_data_flow(&mut self, flow: DataFlow) {
        self.data_flows.push(flow);
        self.touch();
    }
    
    /// Get threats by severity
    pub fn threats_by_severity(&self, severity: ThreatSeverity) -> Vec<&Threat> {
        self.threats.iter().filter(|t| t.severity == severity).collect()
    }
    
    /// Get threats by category
    pub fn threats_by_category(&self, category: ThreatCategory) -> Vec<&Threat> {
        self.threats.iter().filter(|t| t.category == category).collect()
    }
    
    /// Get unmitigated threats
    pub fn unmitigated_threats(&self) -> Vec<&Threat> {
        self.threats.iter().filter(|t| !t.is_mitigated()).collect()
    }
    
    /// Calculate the overall risk score (average of all threat risk scores)
    pub fn overall_risk_score(&self) -> f64 {
        if self.threats.is_empty() {
            return 0.0;
        }
        
        let total_score: u32 = self.threats.iter().map(|t| t.risk_score() as u32).sum();
        total_score as f64 / self.threats.len() as f64
    }
    
    /// Update the threat model's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Represents an asset or resource being protected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier for the asset
    pub id: Uuid,
    
    /// Name of the asset
    pub name: String,
    
    /// Description of the asset
    pub description: String,
    
    /// Sensitivity of the asset
    pub sensitivity: AssetSensitivity,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last update date
    pub updated_at: DateTime<Utc>,
}

/// Sensitivity level of an asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetSensitivity {
    /// Public information
    Public,
    /// Internal information
    Internal,
    /// Confidential information
    Confidential,
    /// Restricted information
    Restricted,
    /// Critical information
    Critical,
}

impl std::fmt::Display for AssetSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "Public"),
            Self::Internal => write!(f, "Internal"),
            Self::Confidential => write!(f, "Confidential"),
            Self::Restricted => write!(f, "Restricted"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

impl Asset {
    /// Create a new asset
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        sensitivity: AssetSensitivity,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            sensitivity,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update the asset's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Represents a trust boundary in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustBoundary {
    /// Unique identifier for the trust boundary
    pub id: Uuid,
    
    /// Name of the trust boundary
    pub name: String,
    
    /// Description of the trust boundary
    pub description: String,
    
    /// Type of the trust boundary
    pub boundary_type: TrustBoundaryType,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last update date
    pub updated_at: DateTime<Utc>,
}

/// Type of trust boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustBoundaryType {
    /// Process boundary
    Process,
    /// Machine boundary
    Machine,
    /// Network boundary
    Network,
    /// Domain boundary
    Domain,
    /// Other boundary type
    Other,
}

impl std::fmt::Display for TrustBoundaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Process => write!(f, "Process"),
            Self::Machine => write!(f, "Machine"),
            Self::Network => write!(f, "Network"),
            Self::Domain => write!(f, "Domain"),
            Self::Other => write!(f, "Other"),
        }
    }
}

impl TrustBoundary {
    /// Create a new trust boundary
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        boundary_type: TrustBoundaryType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            boundary_type,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update the trust boundary's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Represents a data flow in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    /// Unique identifier for the data flow
    pub id: Uuid,
    
    /// Name of the data flow
    pub name: String,
    
    /// Description of the data flow
    pub description: String,
    
    /// Source of the data flow
    pub source: String,
    
    /// Destination of the data flow
    pub destination: String,
    
    /// Data being transferred
    pub data: String,
    
    /// Protocol or method used for the data flow
    pub protocol: Option<String>,
    
    /// Authentication method used for the data flow
    pub authentication: Option<String>,
    
    /// Whether the data flow is encrypted
    pub encrypted: bool,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last update date
    pub updated_at: DateTime<Utc>,
}

impl DataFlow {
    /// Create a new data flow
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        source: impl Into<String>,
        destination: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            source: source.into(),
            destination: destination.into(),
            data: data.into(),
            protocol: None,
            authentication: None,
            encrypted: false,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Set the protocol
    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }
    
    /// Set the authentication method
    pub fn with_authentication(mut self, auth: impl Into<String>) -> Self {
        self.authentication = Some(auth.into());
        self
    }
    
    /// Set whether the data flow is encrypted
    pub fn with_encryption(mut self, encrypted: bool) -> Self {
        self.encrypted = encrypted;
        self
    }
    
    /// Update the data flow's timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Service for managing threat models
#[derive(Debug)]
pub struct ThreatModelingService {
    /// Database connection pool
    db: SqlitePool,
    
    /// Cached threat models
    models: Arc<RwLock<HashMap<Uuid, ThreatModel>>>,
}

impl ThreatModelingService {
    /// Create a new ThreatModelingService
    pub async fn new(db: SqlitePool) -> Result<Self> {
        // Initialize the database tables if they don't exist
        Self::init_db(&db).await?;
        
        // Load models from the database
        let models = Self::load_models_from_db(&db).await?;
        
        let models_map = models.into_iter()
            .map(|model| (model.id, model))
            .collect::<HashMap<_, _>>();
        
        Ok(Self {
            db,
            models: Arc::new(RwLock::new(models_map)),
        })
    }
    
    /// Initialize the database tables
    async fn init_db(db: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS threat_models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                data JSON NOT NULL
            );
            "#,
        )
        .execute(db)
        .await
        .context("Failed to initialize threat modeling database tables")?;
        
        Ok(())
    }
    
    /// Load threat models from the database
    async fn load_models_from_db(db: &SqlitePool) -> Result<Vec<ThreatModel>> {
        let rows = sqlx::query(
            r#"
            SELECT id, data
            FROM threat_models
            "#,
        )
        .fetch_all(db)
        .await
        .context("Failed to load threat models from database")?;
        
        let mut models = Vec::with_capacity(rows.len());
        
        for row in rows {
            let id_str: String = row.get("id");
            let data_json: String = row.get("data");
            
            let id = Uuid::parse_str(&id_str)
                .context(format!("Failed to parse UUID: {}", id_str))?;
            
            let mut model: ThreatModel = serde_json::from_str(&data_json)
                .context(format!("Failed to parse threat model data for ID: {}", id))?;
            
            // Ensure the ID matches
            if model.id != id {
                model.id = id;
            }
            
            models.push(model);
        }
        
        Ok(models)
    }
    
    /// Save a threat model to the database
    async fn save_model_to_db(&self, model: &ThreatModel) -> Result<()> {
        let data_json = serde_json::to_string(model)
            .context(format!("Failed to serialize threat model: {}", model.id))?;
        
        sqlx::query(
            r#"
            INSERT INTO threat_models (id, name, description, created_at, updated_at, data)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT (id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                updated_at = excluded.updated_at,
                data = excluded.data
            "#,
        )
        .bind(model.id.to_string())
        .bind(&model.name)
        .bind(&model.description)
        .bind(model.created_at)
        .bind(model.updated_at)
        .bind(&data_json)
        .execute(&self.db)
        .await
        .context(format!("Failed to save threat model: {}", model.id))?;
        
        Ok(())
    }
    
    /// Create a new threat model
    pub async fn create_model(&self, name: impl Into<String>, description: impl Into<String>) -> Result<Uuid> {
        let model = ThreatModel::new(name, description);
        let id = model.id;
        
        // Save to database
        self.save_model_to_db(&model).await?;
        
        // Add to cache
        {
            let mut models = self.models.write().await;
            models.insert(id, model);
        }
        
        Ok(id)
    }
    
    /// Get a threat model by ID
    pub async fn get_model(&self, id: Uuid) -> Result<ThreatModel> {
        // Check cache first
        {
            let models = self.models.read().await;
            if let Some(model) = models.get(&id) {
                return Ok(model.clone());
            }
        }
        
        // If not in cache, try to load from database
        let row = sqlx::query(
            r#"
            SELECT data
            FROM threat_models
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.db)
        .await
        .context(format!("Failed to query threat model: {}", id))?;
        
        if let Some(row) = row {
            let data_json: String = row.get("data");
            
            let model: ThreatModel = serde_json::from_str(&data_json)
                .context(format!("Failed to parse threat model data for ID: {}", id))?;
            
            // Add to cache
            {
                let mut models = self.models.write().await;
                models.insert(id, model.clone());
            }
            
            Ok(model)
        } else {
            Err(anyhow::anyhow!("Threat model not found: {}", id))
        }
    }
    
    /// Get all threat models
    pub async fn get_all_models(&self) -> Result<Vec<ThreatModel>> {
        let models = self.models.read().await;
        Ok(models.values().cloned().collect())
    }
    
    /// Update a threat model
    pub async fn update_model(&self, model: ThreatModel) -> Result<()> {
        // Save to database
        self.save_model_to_db(&model).await?;
        
        // Update cache
        {
            let mut models = self.models.write().await;
            models.insert(model.id, model);
        }
        
        Ok(())
    }
    
    /// Delete a threat model
    pub async fn delete_model(&self, id: Uuid) -> Result<()> {
        // Delete from database
        sqlx::query(
            r#"
            DELETE FROM threat_models
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.db)
        .await
        .context(format!("Failed to delete threat model: {}", id))?;
        
        // Remove from cache
        {
            let mut models = self.models.write().await;
            models.remove(&id);
        }
        
        Ok(())
    }
    
    /// Add a threat to a model
    pub async fn add_threat(&self, model_id: Uuid, threat: Threat) -> Result<Uuid> {
        let threat_id = threat.id;
        
        // Get the model
        let mut model = self.get_model(model_id).await?;
        
        // Add the threat
        model.add_threat(threat);
        
        // Update the model
        self.update_model(model).await?;
        
        Ok(threat_id)
    }
    
    /// Update a threat in a model
    pub async fn update_threat(&self, model_id: Uuid, threat: Threat) -> Result<()> {
        // Get the model
        let mut model = self.get_model(model_id).await?;
        
        // Find and update the threat
        let threat_index = model.threats.iter().position(|t| t.id == threat.id)
            .ok_or_else(|| anyhow::anyhow!("Threat not found: {}", threat.id))?;
        
        model.threats[threat_index] = threat;
        model.touch();
        
        // Update the model
        self.update_model(model).await?;
        
        Ok(())
    }
    
    /// Delete a threat from a model
    pub async fn delete_threat(&self, model_id: Uuid, threat_id: Uuid) -> Result<()> {
        // Get the model
        let mut model = self.get_model(model_id).await?;
        
        // Find and remove the threat
        let threat_index = model.threats.iter().position(|t| t.id == threat_id)
            .ok_or_else(|| anyhow::anyhow!("Threat not found: {}", threat_id))?;
        
        model.threats.remove(threat_index);
        model.touch();
        
        // Update the model
        self.update_model(model).await?;
        
        Ok(())
    }
    
    /// Generate common threats based on the model's assets and data flows
    pub async fn generate_common_threats(&self, model_id: Uuid) -> Result<Vec<Uuid>> {
        // Get the model
        let mut model = self.get_model(model_id).await?;
        
        let mut threat_ids = Vec::new();
        
        // Generate authentication threats if there are data flows with authentication
        let auth_flows = model.data_flows.iter()
            .filter(|flow| flow.authentication.is_some())
            .collect::<Vec<_>>();
        
        if !auth_flows.is_empty() {
            // Add weak authentication threat
            let threat = Threat::new(
                "Weak Authentication",
                "The system may use weak authentication mechanisms that could be bypassed.",
                ThreatCategory::Authentication,
                ThreatSeverity::High,
            )
            .with_affected_component("Authentication System")
            .with_potential_impact("Unauthorized access to the system")
            .with_likelihood(7)
            .with_mitigation(
                Mitigation::new("Implement strong authentication mechanisms")
                    .with_status(MitigationStatus::NotStarted)
            )
            .with_reference("OWASP", "https://owasp.org/www-project-top-ten/2017/A2_2017-Broken_Authentication");
            
            threat_ids.push(threat.id);
            model.add_threat(threat);
        }
        
        // Generate data exposure threats for sensitive assets
        let sensitive_assets = model.assets.iter()
            .filter(|asset| {
                asset.sensitivity == AssetSensitivity::Confidential ||
                asset.sensitivity == AssetSensitivity::Restricted ||
                asset.sensitivity == AssetSensitivity::Critical
            })
            .collect::<Vec<_>>();
        
        if !sensitive_assets.is_empty() {
            // Add sensitive data exposure threat
            let threat = Threat::new(
                "Sensitive Data Exposure",
                "Sensitive data may be exposed due to lack of encryption or proper access controls.",
                ThreatCategory::DataExposure,
                ThreatSeverity::High,
            )
            .with_affected_component("Data Storage")
            .with_potential_impact("Exposure of sensitive user data")
            .with_likelihood(6)
            .with_mitigation(
                Mitigation::new("Encrypt sensitive data at rest and in transit")
                    .with_status(MitigationStatus::NotStarted)
            )
            .with_reference("OWASP", "https://owasp.org/www-project-top-ten/2017/A3_2017-Sensitive_Data_Exposure");
            
            threat_ids.push(threat.id);
            model.add_threat(threat);
        }
        
        // Generate input validation threats for data flows
        if !model.data_flows.is_empty() {
            // Add injection threat
            let threat = Threat::new(
                "Injection Attacks",
                "The system may be vulnerable to injection attacks if user input is not properly validated.",
                ThreatCategory::InputValidation,
                ThreatSeverity::Critical,
            )
            .with_affected_component("Input Processing")
            .with_potential_impact("Unauthorized data access or system compromise")
            .with_likelihood(8)
            .with_mitigation(
                Mitigation::new("Implement input validation and parameterized queries")
                    .with_status(MitigationStatus::NotStarted)
            )
            .with_reference("OWASP", "https://owasp.org/www-project-top-ten/2017/A1_2017-Injection");
            
            threat_ids.push(threat.id);
            model.add_threat(threat);
        }
        
        // Generate access control threats
        let threat = Threat::new(
            "Broken Access Control",
            "The system may have insufficient access controls allowing unauthorized access to resources.",
            ThreatCategory::AccessControl,
            ThreatSeverity::High,
        )
        .with_affected_component("Access Control System")
        .with_potential_impact("Unauthorized access to resources")
        .with_likelihood(7)
        .with_mitigation(
            Mitigation::new("Implement proper access control checks")
                .with_status(MitigationStatus::NotStarted)
        )
        .with_reference("OWASP", "https://owasp.org/www-project-top-ten/2017/A5_2017-Broken_Access_Control");
        
        threat_ids.push(threat.id);
        model.add_threat(threat);
        
        // Update the model
        self.update_model(model).await?;
        
        Ok(threat_ids)
    }
    
    /// Generate a threat report for a model
    pub async fn generate_report(&self, model_id: Uuid) -> Result<ThreatReport> {
        // Get the model
        let model = self.get_model(model_id).await?;
        
        // Count threats by severity
        let mut severity_counts = HashMap::new();
        for threat in &model.threats {
            *severity_counts.entry(threat.severity).or_insert(0) += 1;
        }
        
        // Count threats by category
        let mut category_counts = HashMap::new();
        for threat in &model.threats {
            *category_counts.entry(threat.category).or_insert(0) += 1;
        }
        
        // Count mitigated vs. unmitigated threats
        let mitigated_count = model.threats.iter().filter(|t| t.is_mitigated()).count();
        let unmitigated_count = model.threats.len() - mitigated_count;
        
        // Calculate overall risk score
        let overall_risk_score = model.overall_risk_score();
        
        // Create the report
        let report = ThreatReport {
            model_id,
            model_name: model.name.clone(),
            total_threats: model.threats.len(),
            severity_counts,
            category_counts,
            mitigated_count,
            unmitigated_count,
            overall_risk_score,
            generated_at: Utc::now(),
        };
        
        Ok(report)
    }
}

/// Represents a threat report for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatReport {
    /// ID of the model
    pub model_id: Uuid,
    
    /// Name of the model
    pub model_name: String,
    
    /// Total number of threats
    pub total_threats: usize,
    
    /// Count of threats by severity
    pub severity_counts: HashMap<ThreatSeverity, usize>,
    
    /// Count of threats by category
    pub category_counts: HashMap<ThreatCategory, usize>,
    
    /// Count of mitigated threats
    pub mitigated_count: usize,
    
    /// Count of unmitigated threats
    pub unmitigated_count: usize,
    
    /// Overall risk score
    pub overall_risk_score: f64,
    
    /// When the report was generated
    pub generated_at: DateTime<Utc>,
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
        let _service = ThreatModelingService::new(db).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_create_and_get_model() -> Result<()> {
        let db = setup_test_db().await;
        let service = ThreatModelingService::new(db).await?;
        
        // Create a model
        let model_id = service.create_model(
            "Test Model",
            "A test threat model",
        ).await?;
        
        // Get the model
        let model = service.get_model(model_id).await?;
        
        assert_eq!(model.name, "Test Model");
        assert_eq!(model.description, "A test threat model");
        assert!(model.threats.is_empty());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_add_and_update_threat() -> Result<()> {
        let db = setup_test_db().await;
        let service = ThreatModelingService::new(db).await?;
        
        // Create a model
        let model_id = service.create_model(
            "Test Model",
            "A test threat model",
        ).await?;
        
        // Create a threat
        let threat = Threat::new(
            "Test Threat",
            "A test threat",
            ThreatCategory::Authentication,
            ThreatSeverity::High,
        );
        
        let threat_id = threat.id;
        
        // Add the threat to the model
        service.add_threat(model_id, threat).await?;
        
        // Get the model
        let model = service.get_model(model_id).await?;
        
        assert_eq!(model.threats.len(), 1);
        assert_eq!(model.threats[0].id, threat_id);
        assert_eq!(model.threats[0].name, "Test Threat");
        
        // Update the threat
        let mut updated_threat = model.threats[0].clone();
        updated_threat.description = "An updated test threat".to_string();
        
        service.update_threat(model_id, updated_threat).await?;
        
        // Get the model again
        let updated_model = service.get_model(model_id).await?;
        
        assert_eq!(updated_model.threats.len(), 1);
        assert_eq!(updated_model.threats[0].id, threat_id);
        assert_eq!(updated_model.threats[0].description, "An updated test threat");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_delete_threat() -> Result<()> {
        let db = setup_test_db().await;
        let service = ThreatModelingService::new(db).await?;
        
        // Create a model
        let model_id = service.create_model(
            "Test Model",
            "A test threat model",
        ).await?;
        
        // Create a threat
        let threat = Threat::new(
            "Test Threat",
            "A test threat",
            ThreatCategory::Authentication,
            ThreatSeverity::High,
        );
        
        let threat_id = threat.id;
        
        // Add the threat to the model
        service.add_threat(model_id, threat).await?;
        
        // Get the model
        let model = service.get_model(model_id).await?;
        
        assert_eq!(model.threats.len(), 1);
        
        // Delete the threat
        service.delete_threat(model_id, threat_id).await?;
        
        // Get the model again
        let updated_model = service.get_model(model_id).await?;
        
        assert_eq!(updated_model.threats.len(), 0);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_generate_common_threats() -> Result<()> {
        let db = setup_test_db().await;
        let service = ThreatModelingService::new(db).await?;
        
        // Create a model
        let model_id = service.create_model(
            "Test Model",
            "A test threat model",
        ).await?;
        
        // Get the model
        let mut model = service.get_model(model_id).await?;
        
        // Add an asset
        let asset = Asset::new(
            "Sensitive Data",
            "User personal information",
            AssetSensitivity::Confidential,
        );
        
        model.add_asset(asset);
        
        // Add a data flow
        let flow = DataFlow::new(
            "User Authentication",
            "User login flow",
            "Client",
            "Server",
            "Credentials",
        )
        .with_authentication("Password");
        
        model.add_data_flow(flow);
        
        // Update the model
        service.update_model(model).await?;
        
        // Generate common threats
        let threat_ids = service.generate_common_threats(model_id).await?;
        
        // Get the model again
        let updated_model = service.get_model(model_id).await?;
        
        // Should have generated multiple threats
        assert!(!threat_ids.is_empty());
        assert_eq!(updated_model.threats.len(), threat_ids.len());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_generate_report() -> Result<()> {
        let db = setup_test_db().await;
        let service = ThreatModelingService::new(db).await?;
        
        // Create a model
        let model_id = service.create_model(
            "Test Model",
            "A test threat model",
        ).await?;
        
        // Add some threats
        let threat1 = Threat::new(
            "Threat 1",
            "A high severity threat",
            ThreatCategory::Authentication,
            ThreatSeverity::High,
        );
        
        let threat2 = Threat::new(
            "Threat 2",
            "A medium severity threat",
            ThreatCategory::DataExposure,
            ThreatSeverity::Medium,
        )
        .with_mitigation(
            Mitigation::new("Implement encryption")
                .with_status(MitigationStatus::Implemented)
        );
        
        service.add_threat(model_id, threat1).await?;
        service.add_threat(model_id, threat2).await?;
        
        // Generate a report
        let report = service.generate_report(model_id).await?;
        
        assert_eq!(report.model_id, model_id);
        assert_eq!(report.model_name, "Test Model");
        assert_eq!(report.total_threats, 2);
        assert_eq!(report.mitigated_count, 1);
        assert_eq!(report.unmitigated_count, 1);
        
        // Check severity counts
        assert_eq!(*report.severity_counts.get(&ThreatSeverity::High).unwrap_or(&0), 1);
        assert_eq!(*report.severity_counts.get(&ThreatSeverity::Medium).unwrap_or(&0), 1);
        
        // Check category counts
        assert_eq!(*report.category_counts.get(&ThreatCategory::Authentication).unwrap_or(&0), 1);
        assert_eq!(*report.category_counts.get(&ThreatCategory::DataExposure).unwrap_or(&0), 1);
        
        Ok(())
    }
}