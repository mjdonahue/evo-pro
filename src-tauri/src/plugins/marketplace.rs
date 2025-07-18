//! Plugin marketplace for discovering and installing plugins
//!
//! This module provides functionality for discovering plugins from remote sources,
//! downloading and installing plugins, updating existing plugins, and managing
//! plugin ratings and reviews.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::{Error, ErrorKind, Result};
use crate::utils::get_data_dir;

use super::interfaces::{PluginMetadata, PluginDependency};
use super::manager::PluginManager;

/// Plugin marketplace source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSource {
    /// Source ID
    pub id: String,
    
    /// Source name
    pub name: String,
    
    /// Source URL
    pub url: String,
    
    /// Source description
    pub description: Option<String>,
    
    /// Whether this is an official source
    pub official: bool,
    
    /// Whether this source is enabled
    pub enabled: bool,
    
    /// Source priority (lower values have higher priority)
    pub priority: i32,
}

/// Plugin marketplace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceEntry {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    
    /// Source ID
    pub source_id: String,
    
    /// Download URL
    pub download_url: String,
    
    /// Plugin size in bytes
    pub size: u64,
    
    /// Plugin release date
    pub release_date: DateTime<Utc>,
    
    /// Plugin download count
    pub download_count: u64,
    
    /// Plugin average rating (0-5)
    pub average_rating: f32,
    
    /// Plugin rating count
    pub rating_count: u32,
    
    /// Plugin screenshots
    pub screenshots: Vec<String>,
    
    /// Plugin changelog
    pub changelog: Option<String>,
    
    /// Whether the plugin is featured
    pub featured: bool,
    
    /// Plugin categories
    pub categories: Vec<String>,
    
    /// Plugin tags
    pub tags: Vec<String>,
}

/// Plugin review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    /// Review ID
    pub id: String,
    
    /// Plugin ID
    pub plugin_id: String,
    
    /// User ID
    pub user_id: String,
    
    /// User name
    pub user_name: String,
    
    /// Rating (0-5)
    pub rating: u8,
    
    /// Review title
    pub title: Option<String>,
    
    /// Review content
    pub content: Option<String>,
    
    /// Review date
    pub date: DateTime<Utc>,
    
    /// Review version
    pub version: String,
}

/// Plugin installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationStatus {
    /// Plugin is being downloaded
    Downloading(u8), // Progress percentage
    
    /// Plugin is being installed
    Installing,
    
    /// Plugin installation is complete
    Complete,
    
    /// Plugin installation failed
    Failed,
}

/// Plugin marketplace manager
pub struct MarketplaceManager {
    /// Marketplace sources
    sources: Vec<MarketplaceSource>,
    
    /// Cached marketplace entries
    entries: HashMap<String, MarketplaceEntry>,
    
    /// Plugin installation statuses
    installation_statuses: HashMap<String, InstallationStatus>,
    
    /// Plugin manager reference
    plugin_manager: Arc<Mutex<PluginManager>>,
    
    /// Download directory
    download_dir: PathBuf,
    
    /// Last refresh time
    last_refresh: Option<DateTime<Utc>>,
}

impl MarketplaceManager {
    /// Create a new marketplace manager
    pub fn new(plugin_manager: Arc<Mutex<PluginManager>>) -> Self {
        // Create the download directory
        let download_dir = get_data_dir().join("downloads").join("plugins");
        std::fs::create_dir_all(&download_dir).unwrap_or_else(|e| {
            tracing::warn!("Failed to create plugin download directory: {}", e);
        });
        
        Self {
            sources: Vec::new(),
            entries: HashMap::new(),
            installation_statuses: HashMap::new(),
            plugin_manager,
            download_dir,
            last_refresh: None,
        }
    }
    
    /// Add a marketplace source
    pub fn add_source(&mut self, source: MarketplaceSource) -> Result<()> {
        // Check if the source already exists
        if self.sources.iter().any(|s| s.id == source.id) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                &format!("Marketplace source with ID {} already exists", source.id)
            ));
        }
        
        // Add the source
        self.sources.push(source);
        
        // Sort sources by priority
        self.sources.sort_by_key(|s| s.priority);
        
        Ok(())
    }
    
    /// Remove a marketplace source
    pub fn remove_source(&mut self, source_id: &str) -> Result<()> {
        // Check if the source exists
        if !self.sources.iter().any(|s| s.id == source_id) {
            return Err(Error::new(
                ErrorKind::NotFound,
                &format!("Marketplace source with ID {} not found", source_id)
            ));
        }
        
        // Remove the source
        self.sources.retain(|s| s.id != source_id);
        
        // Remove entries from this source
        self.entries.retain(|_, e| e.source_id != source_id);
        
        Ok(())
    }
    
    /// Get all marketplace sources
    pub fn get_sources(&self) -> &[MarketplaceSource] {
        &self.sources
    }
    
    /// Enable a marketplace source
    pub fn enable_source(&mut self, source_id: &str) -> Result<()> {
        // Find the source
        let source = self.sources.iter_mut().find(|s| s.id == source_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Marketplace source with ID {} not found", source_id)
            )
        })?;
        
        // Enable the source
        source.enabled = true;
        
        Ok(())
    }
    
    /// Disable a marketplace source
    pub fn disable_source(&mut self, source_id: &str) -> Result<()> {
        // Find the source
        let source = self.sources.iter_mut().find(|s| s.id == source_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Marketplace source with ID {} not found", source_id)
            )
        })?;
        
        // Disable the source
        source.enabled = false;
        
        Ok(())
    }
    
    /// Refresh marketplace entries from all enabled sources
    pub async fn refresh(&mut self) -> Result<()> {
        tracing::info!("Refreshing marketplace entries...");
        
        // Clear existing entries
        self.entries.clear();
        
        // Refresh entries from each enabled source
        for source in self.sources.iter().filter(|s| s.enabled) {
            match self.refresh_source(source).await {
                Ok(_) => {
                    tracing::info!("Refreshed marketplace entries from source: {}", source.name);
                },
                Err(e) => {
                    tracing::warn!("Failed to refresh marketplace entries from source {}: {}", source.name, e);
                }
            }
        }
        
        // Update last refresh time
        self.last_refresh = Some(Utc::now());
        
        tracing::info!("Marketplace refresh completed");
        
        Ok(())
    }
    
    /// Refresh marketplace entries from a specific source
    async fn refresh_source(&mut self, source: &MarketplaceSource) -> Result<()> {
        // In a real implementation, this would make an HTTP request to the source URL
        // to fetch the list of available plugins. For this example, we'll just create
        // some dummy entries.
        
        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Create some dummy entries
        if source.id == "official" {
            // Add some official plugins
            let entries = vec![
                MarketplaceEntry {
                    metadata: PluginMetadata {
                        id: "com.example.markdown-editor".to_string(),
                        name: "Markdown Editor".to_string(),
                        version: "1.0.0".to_string(),
                        description: Some("Advanced markdown editing capabilities".to_string()),
                        author: Some("Evo Design Team".to_string()),
                        homepage: Some("https://example.com/plugins/markdown-editor".to_string()),
                        repository: Some("https://github.com/example/markdown-editor".to_string()),
                        license: Some("MIT".to_string()),
                        min_app_version: Some("1.0.0".to_string()),
                        max_app_version: None,
                        dependencies: Vec::new(),
                        capabilities: vec!["editor".to_string(), "markdown".to_string()],
                        config_schema: None,
                        built_in: false,
                    },
                    source_id: source.id.clone(),
                    download_url: "https://example.com/plugins/markdown-editor/download".to_string(),
                    size: 1024 * 1024, // 1 MB
                    release_date: Utc::now() - chrono::Duration::days(30),
                    download_count: 5000,
                    average_rating: 4.5,
                    rating_count: 120,
                    screenshots: vec![
                        "https://example.com/plugins/markdown-editor/screenshot1.png".to_string(),
                        "https://example.com/plugins/markdown-editor/screenshot2.png".to_string(),
                    ],
                    changelog: Some("Initial release".to_string()),
                    featured: true,
                    categories: vec!["Editor".to_string(), "Productivity".to_string()],
                    tags: vec!["markdown".to_string(), "editor".to_string(), "text".to_string()],
                },
                MarketplaceEntry {
                    metadata: PluginMetadata {
                        id: "com.example.code-formatter".to_string(),
                        name: "Code Formatter".to_string(),
                        version: "1.2.0".to_string(),
                        description: Some("Format code in various languages".to_string()),
                        author: Some("Evo Design Team".to_string()),
                        homepage: Some("https://example.com/plugins/code-formatter".to_string()),
                        repository: Some("https://github.com/example/code-formatter".to_string()),
                        license: Some("MIT".to_string()),
                        min_app_version: Some("1.0.0".to_string()),
                        max_app_version: None,
                        dependencies: Vec::new(),
                        capabilities: vec!["formatter".to_string(), "code".to_string()],
                        config_schema: None,
                        built_in: false,
                    },
                    source_id: source.id.clone(),
                    download_url: "https://example.com/plugins/code-formatter/download".to_string(),
                    size: 2 * 1024 * 1024, // 2 MB
                    release_date: Utc::now() - chrono::Duration::days(15),
                    download_count: 3500,
                    average_rating: 4.2,
                    rating_count: 85,
                    screenshots: vec![
                        "https://example.com/plugins/code-formatter/screenshot1.png".to_string(),
                    ],
                    changelog: Some("Added support for Rust and Go".to_string()),
                    featured: false,
                    categories: vec!["Development".to_string(), "Productivity".to_string()],
                    tags: vec!["code".to_string(), "formatter".to_string(), "development".to_string()],
                },
            ];
            
            // Add entries to the cache
            for entry in entries {
                self.entries.insert(entry.metadata.id.clone(), entry);
            }
        } else if source.id == "community" {
            // Add some community plugins
            let entries = vec![
                MarketplaceEntry {
                    metadata: PluginMetadata {
                        id: "com.community.theme-creator".to_string(),
                        name: "Theme Creator".to_string(),
                        version: "0.9.0".to_string(),
                        description: Some("Create and share custom themes".to_string()),
                        author: Some("Community Developer".to_string()),
                        homepage: Some("https://example.com/plugins/theme-creator".to_string()),
                        repository: Some("https://github.com/community/theme-creator".to_string()),
                        license: Some("MIT".to_string()),
                        min_app_version: Some("1.0.0".to_string()),
                        max_app_version: None,
                        dependencies: Vec::new(),
                        capabilities: vec!["theme".to_string(), "ui".to_string()],
                        config_schema: None,
                        built_in: false,
                    },
                    source_id: source.id.clone(),
                    download_url: "https://example.com/plugins/theme-creator/download".to_string(),
                    size: 500 * 1024, // 500 KB
                    release_date: Utc::now() - chrono::Duration::days(5),
                    download_count: 1200,
                    average_rating: 3.8,
                    rating_count: 45,
                    screenshots: vec![
                        "https://example.com/plugins/theme-creator/screenshot1.png".to_string(),
                    ],
                    changelog: Some("Beta release".to_string()),
                    featured: false,
                    categories: vec!["UI".to_string(), "Customization".to_string()],
                    tags: vec!["theme".to_string(), "ui".to_string(), "customization".to_string()],
                },
            ];
            
            // Add entries to the cache
            for entry in entries {
                self.entries.insert(entry.metadata.id.clone(), entry);
            }
        }
        
        Ok(())
    }
    
    /// Get all marketplace entries
    pub fn get_entries(&self) -> Vec<&MarketplaceEntry> {
        self.entries.values().collect()
    }
    
    /// Get marketplace entries by category
    pub fn get_entries_by_category(&self, category: &str) -> Vec<&MarketplaceEntry> {
        self.entries.values()
            .filter(|e| e.categories.iter().any(|c| c.eq_ignore_ascii_case(category)))
            .collect()
    }
    
    /// Get marketplace entries by tag
    pub fn get_entries_by_tag(&self, tag: &str) -> Vec<&MarketplaceEntry> {
        self.entries.values()
            .filter(|e| e.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }
    
    /// Search marketplace entries
    pub fn search_entries(&self, query: &str) -> Vec<&MarketplaceEntry> {
        let query = query.to_lowercase();
        self.entries.values()
            .filter(|e| {
                e.metadata.name.to_lowercase().contains(&query) ||
                e.metadata.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query)) ||
                e.metadata.author.as_ref().map_or(false, |a| a.to_lowercase().contains(&query)) ||
                e.tags.iter().any(|t| t.to_lowercase().contains(&query)) ||
                e.categories.iter().any(|c| c.to_lowercase().contains(&query))
            })
            .collect()
    }
    
    /// Get featured marketplace entries
    pub fn get_featured_entries(&self) -> Vec<&MarketplaceEntry> {
        self.entries.values()
            .filter(|e| e.featured)
            .collect()
    }
    
    /// Get a marketplace entry by ID
    pub fn get_entry(&self, plugin_id: &str) -> Option<&MarketplaceEntry> {
        self.entries.get(plugin_id)
    }
    
    /// Install a plugin from the marketplace
    pub async fn install_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Get the marketplace entry
        let entry = self.entries.get(plugin_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} not found in marketplace", plugin_id)
            )
        })?;
        
        // Check if the plugin is already installed
        let plugin_manager = self.plugin_manager.lock().unwrap();
        if plugin_manager.is_plugin_loaded(plugin_id) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                &format!("Plugin with ID {} is already installed", plugin_id)
            ));
        }
        drop(plugin_manager);
        
        // Set installation status to downloading
        self.installation_statuses.insert(plugin_id.to_string(), InstallationStatus::Downloading(0));
        
        // Create a download path
        let download_path = self.download_dir.join(format!("{}.zip", plugin_id));
        
        // In a real implementation, this would download the plugin from the download URL
        // For this example, we'll just simulate the download
        
        // Simulate download progress
        for progress in (0..=100).step_by(10) {
            self.installation_statuses.insert(
                plugin_id.to_string(),
                InstallationStatus::Downloading(progress as u8)
            );
            
            // Simulate network delay
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        // Set installation status to installing
        self.installation_statuses.insert(plugin_id.to_string(), InstallationStatus::Installing);
        
        // In a real implementation, this would extract the plugin and install it
        // For this example, we'll just simulate the installation
        
        // Simulate installation delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Register the plugin with the plugin manager
        let mut plugin_manager = self.plugin_manager.lock().unwrap();
        plugin_manager.register_plugin(entry.metadata.clone())?;
        
        // Set installation status to complete
        self.installation_statuses.insert(plugin_id.to_string(), InstallationStatus::Complete);
        
        // Increment download count (in a real implementation, this would be done on the server)
        if let Some(entry) = self.entries.get_mut(plugin_id) {
            entry.download_count += 1;
        }
        
        tracing::info!("Installed plugin: {}", plugin_id);
        
        Ok(())
    }
    
    /// Uninstall a plugin
    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Check if the plugin is installed
        let mut plugin_manager = self.plugin_manager.lock().unwrap();
        if !plugin_manager.is_plugin_loaded(plugin_id) {
            return Err(Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not installed", plugin_id)
            ));
        }
        
        // Unload the plugin
        plugin_manager.unload_plugin(plugin_id).await?;
        
        // In a real implementation, this would also remove the plugin files
        
        tracing::info!("Uninstalled plugin: {}", plugin_id);
        
        Ok(())
    }
    
    /// Update a plugin
    pub async fn update_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Get the marketplace entry
        let entry = self.entries.get(plugin_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} not found in marketplace", plugin_id)
            )
        })?;
        
        // Check if the plugin is installed
        let plugin_manager = self.plugin_manager.lock().unwrap();
        if !plugin_manager.is_plugin_loaded(plugin_id) {
            return Err(Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not installed", plugin_id)
            ));
        }
        
        // Get the installed plugin metadata
        let installed_metadata = plugin_manager.get_plugin(plugin_id).unwrap().metadata().clone();
        drop(plugin_manager);
        
        // Check if an update is available
        if installed_metadata.version == entry.metadata.version {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                &format!("Plugin with ID {} is already up to date", plugin_id)
            ));
        }
        
        // Uninstall the old version
        self.uninstall_plugin(plugin_id).await?;
        
        // Install the new version
        self.install_plugin(plugin_id).await?;
        
        tracing::info!("Updated plugin: {} from {} to {}", plugin_id, installed_metadata.version, entry.metadata.version);
        
        Ok(())
    }
    
    /// Get the installation status of a plugin
    pub fn get_installation_status(&self, plugin_id: &str) -> Option<InstallationStatus> {
        self.installation_statuses.get(plugin_id).copied()
    }
    
    /// Get all available categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories = self.entries.values()
            .flat_map(|e| e.categories.clone())
            .collect::<Vec<_>>();
        
        // Remove duplicates
        categories.sort();
        categories.dedup();
        
        categories
    }
    
    /// Get all available tags
    pub fn get_tags(&self) -> Vec<String> {
        let mut tags = self.entries.values()
            .flat_map(|e| e.tags.clone())
            .collect::<Vec<_>>();
        
        // Remove duplicates
        tags.sort();
        tags.dedup();
        
        tags
    }
    
    /// Initialize with default sources
    pub fn initialize_default_sources(&mut self) -> Result<()> {
        // Add official source
        self.add_source(MarketplaceSource {
            id: "official".to_string(),
            name: "Official Marketplace".to_string(),
            url: "https://marketplace.evo-design.com/api/v1".to_string(),
            description: Some("Official plugin marketplace for Evo Design".to_string()),
            official: true,
            enabled: true,
            priority: 0,
        })?;
        
        // Add community source
        self.add_source(MarketplaceSource {
            id: "community".to_string(),
            name: "Community Plugins".to_string(),
            url: "https://community.evo-design.com/plugins/api/v1".to_string(),
            description: Some("Community-contributed plugins for Evo Design".to_string()),
            official: false,
            enabled: true,
            priority: 10,
        })?;
        
        Ok(())
    }
}

/// Plugin marketplace API for frontend
#[tauri::command]
pub async fn get_marketplace_sources() -> Result<Vec<MarketplaceSource>> {
    // Get the marketplace manager
    let marketplace_manager = get_marketplace_manager();
    let marketplace_manager = marketplace_manager.lock().unwrap();
    
    // Return the sources
    Ok(marketplace_manager.get_sources().to_vec())
}

/// Get marketplace entries
#[tauri::command]
pub async fn get_marketplace_entries() -> Result<Vec<MarketplaceEntry>> {
    // Get the marketplace manager
    let marketplace_manager = get_marketplace_manager();
    let marketplace_manager = marketplace_manager.lock().unwrap();
    
    // Return the entries
    Ok(marketplace_manager.get_entries().into_iter().cloned().collect())
}

/// Search marketplace entries
#[tauri::command]
pub async fn search_marketplace_entries(query: String) -> Result<Vec<MarketplaceEntry>> {
    // Get the marketplace manager
    let marketplace_manager = get_marketplace_manager();
    let marketplace_manager = marketplace_manager.lock().unwrap();
    
    // Search entries
    Ok(marketplace_manager.search_entries(&query).into_iter().cloned().collect())
}

/// Install a plugin
#[tauri::command]
pub async fn install_plugin(plugin_id: String) -> Result<()> {
    // Get the marketplace manager
    let marketplace_manager = get_marketplace_manager();
    let mut marketplace_manager = marketplace_manager.lock().unwrap();
    
    // Install the plugin
    marketplace_manager.install_plugin(&plugin_id).await
}

/// Uninstall a plugin
#[tauri::command]
pub async fn uninstall_plugin(plugin_id: String) -> Result<()> {
    // Get the marketplace manager
    let marketplace_manager = get_marketplace_manager();
    let mut marketplace_manager = marketplace_manager.lock().unwrap();
    
    // Uninstall the plugin
    marketplace_manager.uninstall_plugin(&plugin_id).await
}

/// Update a plugin
#[tauri::command]
pub async fn update_plugin(plugin_id: String) -> Result<()> {
    // Get the marketplace manager
    let marketplace_manager = get_marketplace_manager();
    let mut marketplace_manager = marketplace_manager.lock().unwrap();
    
    // Update the plugin
    marketplace_manager.update_plugin(&plugin_id).await
}

/// Refresh marketplace entries
#[tauri::command]
pub async fn refresh_marketplace() -> Result<()> {
    // Get the marketplace manager
    let marketplace_manager = get_marketplace_manager();
    let mut marketplace_manager = marketplace_manager.lock().unwrap();
    
    // Refresh entries
    marketplace_manager.refresh().await
}

/// Global marketplace manager instance
static MARKETPLACE_MANAGER_INIT: std::sync::Once = std::sync::Once::new();
static mut MARKETPLACE_MANAGER: Option<Arc<Mutex<MarketplaceManager>>> = None;

/// Get the global marketplace manager instance
pub fn get_marketplace_manager() -> Arc<Mutex<MarketplaceManager>> {
    unsafe {
        MARKETPLACE_MANAGER_INIT.call_once(|| {
            // Get the plugin manager
            let plugin_manager = super::get_plugin_manager();
            
            // Create a new marketplace manager
            let mut marketplace_manager = MarketplaceManager::new(plugin_manager);
            
            // Initialize default sources
            if let Err(e) = marketplace_manager.initialize_default_sources() {
                tracing::warn!("Failed to initialize default marketplace sources: {}", e);
            }
            
            MARKETPLACE_MANAGER = Some(Arc::new(Mutex::new(marketplace_manager)));
        });
        
        MARKETPLACE_MANAGER.clone().unwrap()
    }
}