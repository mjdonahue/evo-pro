//! Plugin manager for managing plugin lifecycle
//!
//! This module provides the PluginManager class, which is responsible for
//! managing the lifecycle of plugins, including registration, loading,
//! initialization, and unloading.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use crate::error::{Error, ErrorKind, Result};
use crate::utils::get_data_dir;

use super::interfaces::{Plugin, PluginMetadata, PluginState, PluginType, PluginContext, PluginFactory, PluginEvent, PluginEventListener};
use super::loader::PluginLoader;
use super::registry::PluginRegistry;
use super::security::PluginSecurityManager;
use super::capabilities::CapabilityManager;
use super::versioning::VersionManager;

/// Plugin manager for managing plugin lifecycle
pub struct PluginManager {
    /// Plugin registry
    registry: PluginRegistry,
    
    /// Plugin loader
    loader: PluginLoader,
    
    /// Plugin security manager
    security_manager: PluginSecurityManager,
    
    /// Capability manager
    capability_manager: CapabilityManager,
    
    /// Version manager
    version_manager: VersionManager,
    
    /// Plugin instances
    plugins: HashMap<String, Box<dyn Plugin>>,
    
    /// Plugin event listeners
    event_listeners: Vec<Arc<dyn PluginEventListener>>,
    
    /// Plugin directories
    plugin_dirs: Vec<PathBuf>,
    
    /// Whether discovery has been performed
    discovery_performed: bool,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        // Get the plugin directories
        let mut plugin_dirs = Vec::new();
        
        // Add the built-in plugins directory
        if let Ok(exe_dir) = std::env::current_exe() {
            if let Some(exe_parent) = exe_dir.parent() {
                plugin_dirs.push(exe_parent.join("plugins"));
            }
        }
        
        // Add the user plugins directory
        let data_dir = get_data_dir();
        plugin_dirs.push(data_dir.join("plugins"));
        
        Self {
            registry: PluginRegistry::new(),
            loader: PluginLoader::new(),
            security_manager: PluginSecurityManager::new(),
            capability_manager: CapabilityManager::new(),
            version_manager: VersionManager::new(),
            plugins: HashMap::new(),
            event_listeners: Vec::new(),
            plugin_dirs,
            discovery_performed: false,
        }
    }
    
    /// Register a plugin
    pub fn register_plugin(&mut self, metadata: PluginMetadata) -> Result<()> {
        // Check if the plugin is already registered
        if self.registry.is_registered(&metadata.id) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                &format!("Plugin with ID {} is already registered", metadata.id)
            ));
        }
        
        // Validate the plugin metadata
        self.validate_plugin_metadata(&metadata)?;
        
        // Register the plugin
        self.registry.register_plugin(metadata.clone())?;
        
        tracing::info!("Registered plugin: {}", metadata.id);
        
        Ok(())
    }
    
    /// Load a plugin
    pub async fn load_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Check if the plugin is registered
        if !self.registry.is_registered(plugin_id) {
            return Err(Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not registered", plugin_id)
            ));
        }
        
        // Check if the plugin is already loaded
        if self.plugins.contains_key(plugin_id) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                &format!("Plugin with ID {} is already loaded", plugin_id)
            ));
        }
        
        // Get the plugin metadata
        let metadata = self.registry.get_plugin_metadata(plugin_id)?;
        
        // Check plugin dependencies
        self.check_plugin_dependencies(&metadata)?;
        
        // Load the plugin
        let plugin = self.loader.load_plugin(&metadata).await?;
        
        // Perform security checks
        self.security_manager.check_plugin(&metadata, &plugin)?;
        
        // Register plugin capabilities
        self.capability_manager.register_plugin_capabilities(&metadata)?;
        
        // Add the plugin to the loaded plugins
        self.plugins.insert(plugin_id.to_string(), plugin);
        
        tracing::info!("Loaded plugin: {}", plugin_id);
        
        Ok(())
    }
    
    /// Initialize a plugin
    pub async fn initialize_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Check if the plugin is loaded
        let plugin = self.plugins.get_mut(plugin_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not loaded", plugin_id)
            )
        })?;
        
        // Check if the plugin is already initialized
        if plugin.state() == PluginState::Active {
            return Err(Error::new(
                ErrorKind::InvalidState,
                &format!("Plugin with ID {} is already initialized", plugin_id)
            ));
        }
        
        // Create the plugin context
        let context = self.create_plugin_context(plugin_id)?;
        
        // Initialize the plugin
        plugin.initialize(context).await?;
        
        tracing::info!("Initialized plugin: {}", plugin_id);
        
        Ok(())
    }
    
    /// Start a plugin
    pub async fn start_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Check if the plugin is loaded
        let plugin = self.plugins.get_mut(plugin_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not loaded", plugin_id)
            )
        })?;
        
        // Start the plugin
        plugin.start().await?;
        
        tracing::info!("Started plugin: {}", plugin_id);
        
        Ok(())
    }
    
    /// Stop a plugin
    pub async fn stop_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Check if the plugin is loaded
        let plugin = self.plugins.get_mut(plugin_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not loaded", plugin_id)
            )
        })?;
        
        // Stop the plugin
        plugin.stop().await?;
        
        tracing::info!("Stopped plugin: {}", plugin_id);
        
        Ok(())
    }
    
    /// Unload a plugin
    pub async fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Check if the plugin is loaded
        let mut plugin = self.plugins.remove(plugin_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not loaded", plugin_id)
            )
        })?;
        
        // Unload the plugin
        plugin.unload().await?;
        
        // Unregister plugin capabilities
        let metadata = plugin.metadata();
        self.capability_manager.unregister_plugin_capabilities(&metadata.id)?;
        
        tracing::info!("Unloaded plugin: {}", plugin_id);
        
        Ok(())
    }
    
    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&dyn Plugin> {
        self.plugins.get(plugin_id).map(|p| p.as_ref())
    }
    
    /// Get all loaded plugins
    pub fn get_plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }
    
    /// Get all plugin metadata
    pub fn get_plugin_metadata(&self) -> Vec<PluginMetadata> {
        self.registry.get_all_plugin_metadata()
    }
    
    /// Check if a plugin is loaded
    pub fn is_plugin_loaded(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }
    
    /// Add a plugin event listener
    pub fn add_event_listener(&mut self, listener: Arc<dyn PluginEventListener>) {
        self.event_listeners.push(listener);
    }
    
    /// Remove a plugin event listener
    pub fn remove_event_listener(&mut self, listener: &Arc<dyn PluginEventListener>) {
        self.event_listeners.retain(|l| !Arc::ptr_eq(l, listener));
    }
    
    /// Dispatch a plugin event
    pub async fn dispatch_event(&self, event: PluginEvent) -> Result<()> {
        for listener in &self.event_listeners {
            listener.handle_event(&event).await?;
        }
        
        Ok(())
    }
    
    /// Discover plugins in the plugin directories
    pub fn discover_plugins(&mut self) -> Result<()> {
        if self.discovery_performed {
            return Ok(());
        }
        
        tracing::info!("Discovering plugins...");
        
        // Discover plugins in each plugin directory
        for dir in &self.plugin_dirs {
            self.discover_plugins_in_directory(dir)?;
        }
        
        self.discovery_performed = true;
        
        tracing::info!("Plugin discovery completed");
        
        Ok(())
    }
    
    /// Discover plugins in a directory
    fn discover_plugins_in_directory(&mut self, dir: &Path) -> Result<()> {
        // Check if the directory exists
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }
        
        tracing::debug!("Discovering plugins in directory: {:?}", dir);
        
        // Read the directory entries
        let entries = std::fs::read_dir(dir).map_err(|e| {
            Error::new(
                ErrorKind::IO,
                &format!("Failed to read plugin directory {:?}: {}", dir, e)
            )
        })?;
        
        // Process each entry
        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::new(
                    ErrorKind::IO,
                    &format!("Failed to read directory entry: {}", e)
                )
            })?;
            
            let path = entry.path();
            
            // Check if this is a plugin
            if self.loader.is_plugin_path(&path) {
                // Try to load the plugin metadata
                match self.loader.load_plugin_metadata(&path) {
                    Ok(metadata) => {
                        // Register the plugin
                        if let Err(e) = self.register_plugin(metadata) {
                            tracing::warn!("Failed to register plugin at {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to load plugin metadata from {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Load built-in plugins
    pub fn load_built_in_plugins(&mut self) -> Result<()> {
        tracing::info!("Loading built-in plugins...");
        
        // In a real implementation, we would have a list of built-in plugins
        // and load them here. For now, this is just a placeholder.
        
        tracing::info!("Built-in plugins loaded");
        
        Ok(())
    }
    
    /// Create a plugin context
    fn create_plugin_context(&self, plugin_id: &str) -> Result<PluginContext> {
        // Get the plugin metadata
        let metadata = self.registry.get_plugin_metadata(plugin_id)?;
        
        // Create the plugin data directory
        let data_dir = get_data_dir().join("plugin_data").join(plugin_id);
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            Error::new(
                ErrorKind::IO,
                &format!("Failed to create plugin data directory: {}", e)
            )
        })?;
        
        // Create the plugin cache directory
        let cache_dir = get_data_dir().join("plugin_cache").join(plugin_id);
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            Error::new(
                ErrorKind::IO,
                &format!("Failed to create plugin cache directory: {}", e)
            )
        })?;
        
        // Create the plugin temporary directory
        let temp_dir = std::env::temp_dir().join("evo_design").join("plugins").join(plugin_id);
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            Error::new(
                ErrorKind::IO,
                &format!("Failed to create plugin temporary directory: {}", e)
            )
        })?;
        
        // Create the plugin context
        let context = PluginContext {
            config: HashMap::new(), // In a real implementation, this would be populated with app config
            plugin_config: HashMap::new(), // In a real implementation, this would be populated with plugin config
            data_dir,
            cache_dir,
            temp_dir,
        };
        
        Ok(context)
    }
    
    /// Validate plugin metadata
    fn validate_plugin_metadata(&self, metadata: &PluginMetadata) -> Result<()> {
        // Check required fields
        if metadata.id.is_empty() {
            return Err(Error::new(
                ErrorKind::Validation,
                "Plugin ID cannot be empty"
            ));
        }
        
        if metadata.name.is_empty() {
            return Err(Error::new(
                ErrorKind::Validation,
                "Plugin name cannot be empty"
            ));
        }
        
        if metadata.version.is_empty() {
            return Err(Error::new(
                ErrorKind::Validation,
                "Plugin version cannot be empty"
            ));
        }
        
        // Validate version format
        if !self.version_manager.is_valid_version(&metadata.version) {
            return Err(Error::new(
                ErrorKind::Validation,
                &format!("Invalid plugin version format: {}", metadata.version)
            ));
        }
        
        // Validate application version constraints
        if let Some(min_version) = &metadata.min_app_version {
            if !self.version_manager.is_valid_version(min_version) {
                return Err(Error::new(
                    ErrorKind::Validation,
                    &format!("Invalid minimum application version format: {}", min_version)
                ));
            }
        }
        
        if let Some(max_version) = &metadata.max_app_version {
            if !self.version_manager.is_valid_version(max_version) {
                return Err(Error::new(
                    ErrorKind::Validation,
                    &format!("Invalid maximum application version format: {}", max_version)
                ));
            }
        }
        
        // Validate dependencies
        for dependency in &metadata.dependencies {
            if dependency.id.is_empty() {
                return Err(Error::new(
                    ErrorKind::Validation,
                    "Dependency ID cannot be empty"
                ));
            }
            
            if let Some(min_version) = &dependency.min_version {
                if !self.version_manager.is_valid_version(min_version) {
                    return Err(Error::new(
                        ErrorKind::Validation,
                        &format!("Invalid dependency minimum version format: {}", min_version)
                    ));
                }
            }
            
            if let Some(max_version) = &dependency.max_version {
                if !self.version_manager.is_valid_version(max_version) {
                    return Err(Error::new(
                        ErrorKind::Validation,
                        &format!("Invalid dependency maximum version format: {}", max_version)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Check plugin dependencies
    fn check_plugin_dependencies(&self, metadata: &PluginMetadata) -> Result<()> {
        // Track visited plugins to detect circular dependencies
        let mut visited = HashSet::new();
        visited.insert(metadata.id.clone());
        
        self.check_plugin_dependencies_recursive(metadata, &mut visited)
    }
    
    /// Recursively check plugin dependencies
    fn check_plugin_dependencies_recursive(&self, metadata: &PluginMetadata, visited: &mut HashSet<String>) -> Result<()> {
        for dependency in &metadata.dependencies {
            // Skip optional dependencies that are not available
            if dependency.optional && !self.registry.is_registered(&dependency.id) {
                continue;
            }
            
            // Check if the dependency is registered
            if !self.registry.is_registered(&dependency.id) {
                return Err(Error::new(
                    ErrorKind::DependencyNotFound,
                    &format!("Plugin dependency not found: {}", dependency.id)
                ));
            }
            
            // Get the dependency metadata
            let dep_metadata = self.registry.get_plugin_metadata(&dependency.id)?;
            
            // Check version constraints
            if let Some(min_version) = &dependency.min_version {
                if !self.version_manager.is_version_satisfied(&dep_metadata.version, min_version, None) {
                    return Err(Error::new(
                        ErrorKind::VersionMismatch,
                        &format!(
                            "Plugin dependency version mismatch: {} requires {} >= {}, but found {}",
                            metadata.id, dependency.id, min_version, dep_metadata.version
                        )
                    ));
                }
            }
            
            if let Some(max_version) = &dependency.max_version {
                if !self.version_manager.is_version_satisfied(&dep_metadata.version, None, Some(max_version)) {
                    return Err(Error::new(
                        ErrorKind::VersionMismatch,
                        &format!(
                            "Plugin dependency version mismatch: {} requires {} <= {}, but found {}",
                            metadata.id, dependency.id, max_version, dep_metadata.version
                        )
                    ));
                }
            }
            
            // Check for circular dependencies
            if !visited.insert(dependency.id.clone()) {
                return Err(Error::new(
                    ErrorKind::CircularDependency,
                    &format!("Circular dependency detected: {}", dependency.id)
                ));
            }
            
            // Recursively check dependencies
            self.check_plugin_dependencies_recursive(&dep_metadata, visited)?;
            
            // Remove from visited set when backtracking
            visited.remove(&dependency.id);
        }
        
        Ok(())
    }
}