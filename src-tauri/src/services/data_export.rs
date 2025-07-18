use std::collections::HashMap;
use std::path::Path;
use std::fs::{self, File};
use std::io::Write;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use tracing::{instrument, debug, error};
use uuid::Uuid;

use crate::error::{Result, AppError};
use crate::storage::db::DatabaseManager;
use crate::entities::User;
use crate::entities::participants::Participant;
use crate::entities::messages::Message;
use crate::entities::addresses::Address;
use crate::entities::agents::Agent;
use crate::entities::events::Event;

/// Represents the different categories of data that can be exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportCategory {
    /// Basic user profile information
    Profile,
    /// User preferences and settings
    Preferences,
    /// Messages and conversations
    Messages,
    /// Agents associated with the user
    Agents,
    /// Events and activities
    Events,
    /// Addresses and contact information
    Addresses,
    /// All data categories
    All,
}

/// Configuration for data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// User ID for whom to export data
    pub user_id: Uuid,
    /// Categories of data to export
    pub categories: Vec<ExportCategory>,
    /// Whether to include deleted items
    pub include_deleted: bool,
    /// Date range for data export (start date)
    pub start_date: Option<chrono::DateTime<Utc>>,
    /// Date range for data export (end date)
    pub end_date: Option<chrono::DateTime<Utc>>,
    /// Format for the export (json, csv, etc.)
    pub format: ExportFormat,
}

/// Format options for data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

/// Represents the exported data
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    /// Export metadata
    pub metadata: ExportMetadata,
    /// User profile data
    pub profile: Option<User>,
    /// User preferences
    pub preferences: Option<Value>,
    /// Messages and conversations
    pub messages: Option<Vec<Message>>,
    /// Agents associated with the user
    pub agents: Option<Vec<Agent>>,
    /// Events and activities
    pub events: Option<Vec<Event>>,
    /// Addresses and contact information
    pub addresses: Option<Vec<Address>>,
}

/// Metadata about the export
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// When the export was created
    pub export_date: chrono::DateTime<Utc>,
    /// User ID for whom the data was exported
    pub user_id: Uuid,
    /// Categories included in the export
    pub categories: Vec<ExportCategory>,
    /// Whether deleted items were included
    pub include_deleted: bool,
    /// Date range for the export (start date)
    pub start_date: Option<chrono::DateTime<Utc>>,
    /// Date range for the export (end date)
    pub end_date: Option<chrono::DateTime<Utc>>,
    /// Format of the export
    pub format: ExportFormat,
}

/// Service for exporting user data
pub struct DataExportService<'a> {
    db: &'a DatabaseManager,
}

impl<'a> DataExportService<'a> {
    /// Create a new DataExportService
    pub fn new(db: &'a DatabaseManager) -> Self {
        Self { db }
    }

    /// Export user data according to the provided configuration
    #[instrument(skip(self, config), err)]
    pub async fn export_data(&self, config: &ExportConfig) -> Result<ExportData> {
        debug!("Exporting data for user: {}", config.user_id);

        // Check if user exists
        let user = match self.db.get_user_by_id(&config.user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", config.user_id))),
        };

        let mut export_data = ExportData {
            metadata: ExportMetadata {
                export_date: Utc::now(),
                user_id: config.user_id,
                categories: config.categories.clone(),
                include_deleted: config.include_deleted,
                start_date: config.start_date,
                end_date: config.end_date,
                format: config.format.clone(),
            },
            profile: None,
            preferences: None,
            messages: None,
            agents: None,
            events: None,
            addresses: None,
        };

        // Export data based on selected categories
        for category in &config.categories {
            match category {
                ExportCategory::Profile | ExportCategory::All => {
                    export_data.profile = Some(user.clone());
                }
                ExportCategory::Preferences | ExportCategory::All => {
                    if let Some(prefs) = &user.preferences {
                        export_data.preferences = Some(prefs.0.clone());
                    }
                }
                ExportCategory::Messages | ExportCategory::All => {
                    export_data.messages = Some(self.export_messages(&config).await?);
                }
                ExportCategory::Agents | ExportCategory::All => {
                    export_data.agents = Some(self.export_agents(&config).await?);
                }
                ExportCategory::Events | ExportCategory::All => {
                    export_data.events = Some(self.export_events(&config).await?);
                }
                ExportCategory::Addresses | ExportCategory::All => {
                    export_data.addresses = Some(self.export_addresses(&config).await?);
                }
            }
        }

        Ok(export_data)
    }

    /// Export messages for a user
    async fn export_messages(&self, config: &ExportConfig) -> Result<Vec<Message>> {
        debug!("Exporting messages for user: {}", config.user_id);
        
        // Get all conversations where the user is a participant
        let conversations = self.db.get_conversations_by_user_id(&config.user_id).await?;
        let mut all_messages = Vec::new();

        for conversation in conversations {
            let mut messages = self.db.get_messages_by_conversation_id(
                &conversation.id,
                config.include_deleted,
                config.start_date,
                config.end_date,
            ).await?;
            all_messages.append(&mut messages);
        }

        Ok(all_messages)
    }

    /// Export agents for a user
    async fn export_agents(&self, config: &ExportConfig) -> Result<Vec<Agent>> {
        debug!("Exporting agents for user: {}", config.user_id);
        
        // Get agents operated by the user
        let agents = self.db.get_agents_by_operator_id(&config.user_id).await?;
        
        Ok(agents)
    }

    /// Export events for a user
    async fn export_events(&self, config: &ExportConfig) -> Result<Vec<Event>> {
        debug!("Exporting events for user: {}", config.user_id);
        
        // Get events created by or related to the user
        let events = self.db.get_events_by_user_id(
            &config.user_id,
            config.include_deleted,
            config.start_date,
            config.end_date,
        ).await?;
        
        Ok(events)
    }

    /// Export addresses for a user
    async fn export_addresses(&self, config: &ExportConfig) -> Result<Vec<Address>> {
        debug!("Exporting addresses for user: {}", config.user_id);
        
        // Get addresses associated with the user
        let addresses = self.db.get_addresses_by_user_id(&config.user_id).await?;
        
        Ok(addresses)
    }

    /// Save the exported data to a file
    #[instrument(skip(self, export_data, path), err)]
    pub async fn save_to_file(&self, export_data: &ExportData, path: &Path) -> Result<()> {
        debug!("Saving exported data to file: {:?}", path);
        
        match export_data.metadata.format {
            ExportFormat::Json => self.save_as_json(export_data, path).await,
            ExportFormat::Csv => self.save_as_csv(export_data, path).await,
        }
    }

    /// Save the exported data as JSON
    async fn save_as_json(&self, export_data: &ExportData, path: &Path) -> Result<()> {
        let json_data = serde_json::to_string_pretty(export_data)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;
        
        fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new("")))?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;
        
        Ok(())
    }

    /// Save the exported data as CSV
    async fn save_as_csv(&self, export_data: &ExportData, path: &Path) -> Result<()> {
        fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new("")))?;
        
        // Create a directory for CSV files
        let dir_path = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or_else(|| Path::new("")).to_path_buf()
        };
        
        // Save metadata
        let metadata_path = dir_path.join("metadata.csv");
        let mut metadata_file = File::create(&metadata_path)?;
        writeln!(metadata_file, "export_date,user_id,include_deleted")?;
        writeln!(
            metadata_file, 
            "{},{},{}", 
            export_data.metadata.export_date, 
            export_data.metadata.user_id,
            export_data.metadata.include_deleted
        )?;
        
        // Save profile if available
        if let Some(profile) = &export_data.profile {
            let profile_path = dir_path.join("profile.csv");
            let mut profile_file = File::create(&profile_path)?;
            writeln!(
                profile_file, 
                "id,email,username,display_name,first_name,last_name,status,primary_role,created_at,updated_at"
            )?;
            writeln!(
                profile_file,
                "{},{},{},{},{},{},{:?},{:?},{},{}",
                profile.id,
                profile.email.as_deref().unwrap_or(""),
                profile.username.as_deref().unwrap_or(""),
                profile.display_name,
                profile.first_name.as_deref().unwrap_or(""),
                profile.last_name.as_deref().unwrap_or(""),
                profile.status,
                profile.primary_role,
                profile.created_at,
                profile.updated_at
            )?;
        }
        
        // Save messages if available
        if let Some(messages) = &export_data.messages {
            let messages_path = dir_path.join("messages.csv");
            let mut messages_file = File::create(&messages_path)?;
            writeln!(
                messages_file,
                "id,conversation_id,sender_id,content,created_at"
            )?;
            
            for message in messages {
                writeln!(
                    messages_file,
                    "{},{},{},{},{}",
                    message.id,
                    message.conversation_id,
                    message.sender_id,
                    message.content.replace(",", "\\,"),
                    message.created_at
                )?;
            }
        }
        
        // Save agents if available
        if let Some(agents) = &export_data.agents {
            let agents_path = dir_path.join("agents.csv");
            let mut agents_file = File::create(&agents_path)?;
            writeln!(
                agents_file,
                "id,name,description,status,created_at"
            )?;
            
            for agent in agents {
                writeln!(
                    agents_file,
                    "{},{},{},{:?},{}",
                    agent.id,
                    agent.name.replace(",", "\\,"),
                    agent.description.as_deref().unwrap_or("").replace(",", "\\,"),
                    agent.status,
                    agent.created_at
                )?;
            }
        }
        
        // Save events if available
        if let Some(events) = &export_data.events {
            let events_path = dir_path.join("events.csv");
            let mut events_file = File::create(&events_path)?;
            writeln!(
                events_file,
                "id,title,description,created_at"
            )?;
            
            for event in events {
                writeln!(
                    events_file,
                    "{},{},{},{}",
                    event.id,
                    event.title.replace(",", "\\,"),
                    event.description.as_deref().unwrap_or("").replace(",", "\\,"),
                    event.created_at
                )?;
            }
        }
        
        // Save addresses if available
        if let Some(addresses) = &export_data.addresses {
            let addresses_path = dir_path.join("addresses.csv");
            let mut addresses_file = File::create(&addresses_path)?;
            writeln!(
                addresses_file,
                "id,address_type,street,city,state,postal_code,country,created_at"
            )?;
            
            for address in addresses {
                writeln!(
                    addresses_file,
                    "{},{:?},{},{},{},{},{},{}",
                    address.id,
                    address.address_type,
                    address.street.as_deref().unwrap_or("").replace(",", "\\,"),
                    address.city.as_deref().unwrap_or("").replace(",", "\\,"),
                    address.state.as_deref().unwrap_or("").replace(",", "\\,"),
                    address.postal_code.as_deref().unwrap_or("").replace(",", "\\,"),
                    address.country.as_deref().unwrap_or("").replace(",", "\\,"),
                    address.created_at
                )?;
            }
        }
        
        Ok(())
    }
}

// Tauri command for exporting user data
#[tauri::command]
pub async fn export_user_data(
    user_id: String,
    categories: Vec<String>,
    include_deleted: bool,
    start_date: Option<String>,
    end_date: Option<String>,
    format: String,
    export_path: String,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<String, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
    
    // Convert string categories to ExportCategory enum
    let categories = categories.iter().map(|c| {
        match c.as_str() {
            "profile" => ExportCategory::Profile,
            "preferences" => ExportCategory::Preferences,
            "messages" => ExportCategory::Messages,
            "agents" => ExportCategory::Agents,
            "events" => ExportCategory::Events,
            "addresses" => ExportCategory::Addresses,
            "all" => ExportCategory::All,
            _ => ExportCategory::Profile, // Default to profile
        }
    }).collect::<Vec<_>>();
    
    // Parse dates if provided
    let start_date = start_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
    let end_date = end_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
    
    // Determine export format
    let format = match format.as_str() {
        "csv" => ExportFormat::Csv,
        _ => ExportFormat::Json,
    };
    
    let config = ExportConfig {
        user_id,
        categories,
        include_deleted,
        start_date,
        end_date,
        format,
    };
    
    let export_service = DataExportService::new(&db);
    
    match export_service.export_data(&config).await {
        Ok(export_data) => {
            let path = Path::new(&export_path);
            match export_service.save_to_file(&export_data, path).await {
                Ok(()) => Ok(format!("Data exported successfully to {}", export_path)),
                Err(e) => Err(format!("Failed to save exported data: {}", e)),
            }
        },
        Err(e) => Err(format!("Failed to export data: {}", e)),
    }
}