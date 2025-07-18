//! Plugin registry for storing and retrieving plugin metadata
//!
//! This module provides the PluginRegistry class, which is responsible for
//! storing and retrieving plugin metadata.

use std::collections::HashMap;
use crate::error::{Error, ErrorKind, Result};
use super::interfaces::PluginMetadata;

/// Plugin registry for storing and retrieving plugin metadata
pub struct PluginRegistry {
    /// Registered plugins by ID
    plugins: HashMap<String, PluginMetadata>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin
    pub fn register_plugin(&mut self, metadata: PluginMetadata) -> Result<()> {
        // Check if the plugin is already registered
        if self.plugins.contains_key(&metadata.id) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                &format!("Plugin with ID {} is already registered", metadata.id)
            ));
        }
        
        // Register the plugin
        self.plugins.insert(metadata.id.clone(), metadata);
        
        Ok(())
    }
    
    /// Unregister a plugin
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Check if the plugin is registered
        if !self.plugins.contains_key(plugin_id) {
            return Err(Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not registered", plugin_id)
            ));
        }
        
        // Unregister the plugin
        self.plugins.remove(plugin_id);
        
        Ok(())
    }
    
    /// Check if a plugin is registered
    pub fn is_registered(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }
    
    /// Get plugin metadata
    pub fn get_plugin_metadata(&self, plugin_id: &str) -> Result<PluginMetadata> {
        // Check if the plugin is registered
        if !self.plugins.contains_key(plugin_id) {
            return Err(Error::new(
                ErrorKind::NotFound,
                &format!("Plugin with ID {} is not registered", plugin_id)
            ));
        }
        
        // Get the plugin metadata
        Ok(self.plugins.get(plugin_id).unwrap().clone())
    }
    
    /// Get all plugin metadata
    pub fn get_all_plugin_metadata(&self) -> Vec<PluginMetadata> {
        self.plugins.values().cloned().collect()
    }
    
    /// Get the number of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
    
    /// Get all plugin IDs
    pub fn get_plugin_ids(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
    
    /// Get built-in plugins
    pub fn get_built_in_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values()
            .filter(|m| m.built_in)
            .cloned()
            .collect()
    }
    
    /// Get external plugins
    pub fn get_external_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values()
            .filter(|m| !m.built_in)
            .cloned()
            .collect()
    }
    
    /// Find plugins by capability
    pub fn find_plugins_by_capability(&self, capability: &str) -> Vec<PluginMetadata> {
        self.plugins.values()
            .filter(|m| m.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect()
    }
    
    /// Find plugins by author
    pub fn find_plugins_by_author(&self, author: &str) -> Vec<PluginMetadata> {
        self.plugins.values()
            .filter(|m| m.author.as_ref().map_or(false, |a| a == author))
            .cloned()
            .collect()
    }
    
    /// Clear the registry
    pub fn clear(&mut self) {
        self.plugins.clear();
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_metadata(id: &str) -> PluginMetadata {
        PluginMetadata {
            id: id.to_string(),
            name: format!("Test Plugin {}", id),
            version: "1.0.0".to_string(),
            description: Some(format!("Test plugin {}", id)),
            author: Some("Test Author".to_string()),
            homepage: None,
            repository: None,
            license: Some("MIT".to_string()),
            min_app_version: None,
            max_app_version: None,
            dependencies: Vec::new(),
            capabilities: vec!["test".to_string()],
            config_schema: None,
            built_in: false,
        }
    }
    
    #[test]
    fn test_register_plugin() {
        let mut registry = PluginRegistry::new();
        let metadata = create_test_metadata("test1");
        
        assert!(registry.register_plugin(metadata.clone()).is_ok());
        assert!(registry.is_registered(&metadata.id));
        assert_eq!(registry.plugin_count(), 1);
    }
    
    #[test]
    fn test_register_duplicate_plugin() {
        let mut registry = PluginRegistry::new();
        let metadata = create_test_metadata("test1");
        
        assert!(registry.register_plugin(metadata.clone()).is_ok());
        assert!(registry.register_plugin(metadata.clone()).is_err());
    }
    
    #[test]
    fn test_unregister_plugin() {
        let mut registry = PluginRegistry::new();
        let metadata = create_test_metadata("test1");
        
        assert!(registry.register_plugin(metadata.clone()).is_ok());
        assert!(registry.unregister_plugin(&metadata.id).is_ok());
        assert!(!registry.is_registered(&metadata.id));
        assert_eq!(registry.plugin_count(), 0);
    }
    
    #[test]
    fn test_unregister_nonexistent_plugin() {
        let mut registry = PluginRegistry::new();
        
        assert!(registry.unregister_plugin("nonexistent").is_err());
    }
    
    #[test]
    fn test_get_plugin_metadata() {
        let mut registry = PluginRegistry::new();
        let metadata = create_test_metadata("test1");
        
        assert!(registry.register_plugin(metadata.clone()).is_ok());
        
        let retrieved = registry.get_plugin_metadata(&metadata.id).unwrap();
        assert_eq!(retrieved.id, metadata.id);
        assert_eq!(retrieved.name, metadata.name);
        assert_eq!(retrieved.version, metadata.version);
    }
    
    #[test]
    fn test_get_nonexistent_plugin_metadata() {
        let registry = PluginRegistry::new();
        
        assert!(registry.get_plugin_metadata("nonexistent").is_err());
    }
    
    #[test]
    fn test_get_all_plugin_metadata() {
        let mut registry = PluginRegistry::new();
        let metadata1 = create_test_metadata("test1");
        let metadata2 = create_test_metadata("test2");
        
        assert!(registry.register_plugin(metadata1.clone()).is_ok());
        assert!(registry.register_plugin(metadata2.clone()).is_ok());
        
        let all_metadata = registry.get_all_plugin_metadata();
        assert_eq!(all_metadata.len(), 2);
        assert!(all_metadata.iter().any(|m| m.id == metadata1.id));
        assert!(all_metadata.iter().any(|m| m.id == metadata2.id));
    }
    
    #[test]
    fn test_find_plugins_by_capability() {
        let mut registry = PluginRegistry::new();
        let mut metadata1 = create_test_metadata("test1");
        let mut metadata2 = create_test_metadata("test2");
        
        metadata1.capabilities = vec!["capability1".to_string(), "capability2".to_string()];
        metadata2.capabilities = vec!["capability2".to_string(), "capability3".to_string()];
        
        assert!(registry.register_plugin(metadata1.clone()).is_ok());
        assert!(registry.register_plugin(metadata2.clone()).is_ok());
        
        let plugins = registry.find_plugins_by_capability("capability1");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, metadata1.id);
        
        let plugins = registry.find_plugins_by_capability("capability2");
        assert_eq!(plugins.len(), 2);
        
        let plugins = registry.find_plugins_by_capability("capability3");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, metadata2.id);
        
        let plugins = registry.find_plugins_by_capability("nonexistent");
        assert_eq!(plugins.len(), 0);
    }
    
    #[test]
    fn test_built_in_plugins() {
        let mut registry = PluginRegistry::new();
        let mut metadata1 = create_test_metadata("test1");
        let metadata2 = create_test_metadata("test2");
        
        metadata1.built_in = true;
        
        assert!(registry.register_plugin(metadata1.clone()).is_ok());
        assert!(registry.register_plugin(metadata2.clone()).is_ok());
        
        let built_in = registry.get_built_in_plugins();
        assert_eq!(built_in.len(), 1);
        assert_eq!(built_in[0].id, metadata1.id);
        
        let external = registry.get_external_plugins();
        assert_eq!(external.len(), 1);
        assert_eq!(external[0].id, metadata2.id);
    }
}