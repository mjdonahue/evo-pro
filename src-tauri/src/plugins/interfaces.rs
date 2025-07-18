//! Plugin interfaces and abstractions
//!
//! This module defines the core interfaces that plugins must implement,
//! as well as related types and structures for plugin metadata,
//! capabilities, and lifecycle management.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::Result;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: String,
    
    /// Human-readable name of the plugin
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Plugin description
    pub description: Option<String>,
    
    /// Plugin author
    pub author: Option<String>,
    
    /// Plugin homepage URL
    pub homepage: Option<String>,
    
    /// Plugin repository URL
    pub repository: Option<String>,
    
    /// Plugin license
    pub license: Option<String>,
    
    /// Minimum application version required
    pub min_app_version: Option<String>,
    
    /// Maximum application version supported
    pub max_app_version: Option<String>,
    
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
    
    /// Plugin capabilities
    pub capabilities: Vec<String>,
    
    /// Plugin configuration schema
    pub config_schema: Option<serde_json::Value>,
    
    /// Whether this is a built-in plugin
    pub built_in: bool,
}

/// Plugin dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Plugin ID
    pub id: String,
    
    /// Minimum version required
    pub min_version: Option<String>,
    
    /// Maximum version supported
    pub max_version: Option<String>,
    
    /// Whether this is an optional dependency
    pub optional: bool,
}

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is registered but not loaded
    Registered,
    
    /// Plugin is loaded but not initialized
    Loaded,
    
    /// Plugin is initialized and active
    Active,
    
    /// Plugin is disabled
    Disabled,
    
    /// Plugin failed to load or initialize
    Failed,
}

/// Plugin type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    /// Built-in plugin
    BuiltIn,
    
    /// Native plugin (compiled Rust code)
    Native,
    
    /// WebAssembly plugin
    Wasm,
    
    /// JavaScript plugin
    JavaScript,
    
    /// Python plugin
    Python,
}

/// Plugin context provided to plugins during initialization
pub struct PluginContext {
    /// Application configuration
    pub config: HashMap<String, serde_json::Value>,
    
    /// Plugin-specific configuration
    pub plugin_config: HashMap<String, serde_json::Value>,
    
    /// Plugin data directory
    pub data_dir: PathBuf,
    
    /// Plugin cache directory
    pub cache_dir: PathBuf,
    
    /// Plugin temporary directory
    pub temp_dir: PathBuf,
}

/// Core plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    async fn initialize(&mut self, context: PluginContext) -> Result<()>;
    
    /// Start the plugin
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the plugin
    async fn stop(&mut self) -> Result<()>;
    
    /// Unload the plugin
    async fn unload(&mut self) -> Result<()>;
    
    /// Get the plugin state
    fn state(&self) -> PluginState;
    
    /// Get the plugin type
    fn plugin_type(&self) -> PluginType;
    
    /// Check if the plugin has a specific capability
    fn has_capability(&self, capability: &str) -> bool {
        self.metadata().capabilities.contains(&capability.to_string())
    }
    
    /// Get the plugin configuration schema
    fn config_schema(&self) -> Option<&serde_json::Value> {
        self.metadata().config_schema.as_ref()
    }
}

/// Plugin factory for creating plugin instances
#[async_trait]
pub trait PluginFactory: Send + Sync {
    /// Create a new plugin instance
    async fn create_plugin(&self) -> Result<Box<dyn Plugin>>;
    
    /// Get the plugin metadata
    fn metadata(&self) -> &PluginMetadata;
}

/// Plugin event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    /// Plugin ID
    pub plugin_id: String,
    
    /// Event type
    pub event_type: String,
    
    /// Event data
    pub data: serde_json::Value,
    
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Plugin event listener
#[async_trait]
pub trait PluginEventListener: Send + Sync {
    /// Handle a plugin event
    async fn handle_event(&self, event: &PluginEvent) -> Result<()>;
}