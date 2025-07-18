//! Plugin loader for loading plugins from various sources
//!
//! This module provides the PluginLoader class, which is responsible for
//! loading plugins from various sources (built-in, native, WASM, etc.).

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use async_trait::async_trait;
use crate::error::{Error, ErrorKind, Result};

use super::interfaces::{Plugin, PluginMetadata, PluginState, PluginType, PluginFactory};

/// Plugin loader for loading plugins from various sources
pub struct PluginLoader {
    /// Plugin factories by ID
    factories: HashMap<String, Box<dyn PluginFactory>>,
    
    /// Plugin paths by ID
    plugin_paths: HashMap<String, PathBuf>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            plugin_paths: HashMap::new(),
        }
    }
    
    /// Register a plugin factory
    pub fn register_factory(&mut self, factory: Box<dyn PluginFactory>) -> Result<()> {
        let metadata = factory.metadata();
        
        // Check if the factory is already registered
        if self.factories.contains_key(&metadata.id) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                &format!("Plugin factory with ID {} is already registered", metadata.id)
            ));
        }
        
        // Register the factory
        self.factories.insert(metadata.id.clone(), factory);
        
        Ok(())
    }
    
    /// Check if a path is a plugin
    pub fn is_plugin_path(&self, path: &Path) -> bool {
        // Check if the path is a directory
        if path.is_dir() {
            // Check if the directory contains a manifest file
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                return true;
            }
            
            // Check if the directory contains a plugin.json file
            let plugin_json_path = path.join("plugin.json");
            if plugin_json_path.exists() {
                return true;
            }
        }
        
        // Check if the path is a file with a supported extension
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let extension_str = extension.to_string_lossy().to_lowercase();
                return match extension_str.as_str() {
                    "so" | "dll" | "dylib" => true, // Native plugins
                    "wasm" => true, // WebAssembly plugins
                    "js" => true, // JavaScript plugins
                    "py" => true, // Python plugins
                    _ => false,
                };
            }
        }
        
        false
    }
    
    /// Load plugin metadata from a path
    pub fn load_plugin_metadata(&self, path: &Path) -> Result<PluginMetadata> {
        // Check if the path is a directory
        if path.is_dir() {
            // Try to load from manifest.json
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                return self.load_plugin_metadata_from_json(&manifest_path);
            }
            
            // Try to load from plugin.json
            let plugin_json_path = path.join("plugin.json");
            if plugin_json_path.exists() {
                return self.load_plugin_metadata_from_json(&plugin_json_path);
            }
        }
        
        // Check if the path is a file with a supported extension
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let extension_str = extension.to_string_lossy().to_lowercase();
                match extension_str.as_str() {
                    "so" | "dll" | "dylib" => {
                        // Load metadata from native plugin
                        return self.load_plugin_metadata_from_native(path);
                    },
                    "wasm" => {
                        // Load metadata from WebAssembly plugin
                        return self.load_plugin_metadata_from_wasm(path);
                    },
                    "js" => {
                        // Load metadata from JavaScript plugin
                        return self.load_plugin_metadata_from_js(path);
                    },
                    "py" => {
                        // Load metadata from Python plugin
                        return self.load_plugin_metadata_from_python(path);
                    },
                    _ => {}
                }
            }
        }
        
        Err(Error::new(
            ErrorKind::InvalidArgument,
            &format!("Unsupported plugin path: {:?}", path)
        ))
    }
    
    /// Load plugin metadata from a JSON file
    fn load_plugin_metadata_from_json(&self, path: &Path) -> Result<PluginMetadata> {
        // Read the file
        let json_str = std::fs::read_to_string(path).map_err(|e| {
            Error::new(
                ErrorKind::IO,
                &format!("Failed to read plugin metadata file: {}", e)
            )
        })?;
        
        // Parse the JSON
        let metadata: PluginMetadata = serde_json::from_str(&json_str).map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to parse plugin metadata: {}", e)
            )
        })?;
        
        Ok(metadata)
    }
    
    /// Load plugin metadata from a native plugin
    fn load_plugin_metadata_from_native(&self, path: &Path) -> Result<PluginMetadata> {
        // In a real implementation, we would load the native library and call a function
        // to get the plugin metadata. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading metadata from native plugins is not implemented yet"
        ))
    }
    
    /// Load plugin metadata from a WebAssembly plugin
    fn load_plugin_metadata_from_wasm(&self, path: &Path) -> Result<PluginMetadata> {
        // In a real implementation, we would load the WebAssembly module and call a function
        // to get the plugin metadata. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading metadata from WebAssembly plugins is not implemented yet"
        ))
    }
    
    /// Load plugin metadata from a JavaScript plugin
    fn load_plugin_metadata_from_js(&self, path: &Path) -> Result<PluginMetadata> {
        // In a real implementation, we would execute the JavaScript code and get the plugin
        // metadata. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading metadata from JavaScript plugins is not implemented yet"
        ))
    }
    
    /// Load plugin metadata from a Python plugin
    fn load_plugin_metadata_from_python(&self, path: &Path) -> Result<PluginMetadata> {
        // In a real implementation, we would execute the Python code and get the plugin
        // metadata. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading metadata from Python plugins is not implemented yet"
        ))
    }
    
    /// Load a plugin
    pub async fn load_plugin(&mut self, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // Check if we have a factory for this plugin
        if let Some(factory) = self.factories.get(&metadata.id) {
            // Create a plugin instance
            return factory.create_plugin().await;
        }
        
        // Check if we have a path for this plugin
        if let Some(path) = self.plugin_paths.get(&metadata.id) {
            // Load the plugin from the path
            return self.load_plugin_from_path(path, metadata).await;
        }
        
        Err(Error::new(
            ErrorKind::NotFound,
            &format!("No factory or path found for plugin: {}", metadata.id)
        ))
    }
    
    /// Load a plugin from a path
    async fn load_plugin_from_path(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // Check if the path is a directory
        if path.is_dir() {
            // Try to load from the directory
            return self.load_plugin_from_directory(path, metadata).await;
        }
        
        // Check if the path is a file with a supported extension
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let extension_str = extension.to_string_lossy().to_lowercase();
                match extension_str.as_str() {
                    "so" | "dll" | "dylib" => {
                        // Load from native plugin
                        return self.load_plugin_from_native(path, metadata).await;
                    },
                    "wasm" => {
                        // Load from WebAssembly plugin
                        return self.load_plugin_from_wasm(path, metadata).await;
                    },
                    "js" => {
                        // Load from JavaScript plugin
                        return self.load_plugin_from_js(path, metadata).await;
                    },
                    "py" => {
                        // Load from Python plugin
                        return self.load_plugin_from_python(path, metadata).await;
                    },
                    _ => {}
                }
            }
        }
        
        Err(Error::new(
            ErrorKind::InvalidArgument,
            &format!("Unsupported plugin path: {:?}", path)
        ))
    }
    
    /// Load a plugin from a directory
    async fn load_plugin_from_directory(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // In a real implementation, we would determine the plugin type from the directory
        // contents and load it accordingly. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading plugins from directories is not implemented yet"
        ))
    }
    
    /// Load a plugin from a native library
    async fn load_plugin_from_native(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // In a real implementation, we would load the native library and create a plugin
        // instance. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading native plugins is not implemented yet"
        ))
    }
    
    /// Load a plugin from a WebAssembly module
    async fn load_plugin_from_wasm(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // In a real implementation, we would load the WebAssembly module and create a plugin
        // instance. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading WebAssembly plugins is not implemented yet"
        ))
    }
    
    /// Load a plugin from a JavaScript file
    async fn load_plugin_from_js(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // In a real implementation, we would execute the JavaScript code and create a plugin
        // instance. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading JavaScript plugins is not implemented yet"
        ))
    }
    
    /// Load a plugin from a Python file
    async fn load_plugin_from_python(&self, path: &Path, metadata: &PluginMetadata) -> Result<Box<dyn Plugin>> {
        // In a real implementation, we would execute the Python code and create a plugin
        // instance. For now, we'll just return an error.
        Err(Error::new(
            ErrorKind::NotImplemented,
            "Loading Python plugins is not implemented yet"
        ))
    }
    
    /// Register a plugin path
    pub fn register_plugin_path(&mut self, plugin_id: &str, path: PathBuf) {
        self.plugin_paths.insert(plugin_id.to_string(), path);
    }
    
    /// Get a plugin path
    pub fn get_plugin_path(&self, plugin_id: &str) -> Option<&PathBuf> {
        self.plugin_paths.get(plugin_id)
    }
    
    /// Clear all registered factories and paths
    pub fn clear(&mut self) {
        self.factories.clear();
        self.plugin_paths.clear();
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in plugin factory
pub struct BuiltInPluginFactory<T: Plugin + Default> {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// Phantom data for the plugin type
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Plugin + Default> BuiltInPluginFactory<T> {
    /// Create a new built-in plugin factory
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: Plugin + Default + 'static> PluginFactory for BuiltInPluginFactory<T> {
    /// Create a new plugin instance
    async fn create_plugin(&self) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(T::default()))
    }
    
    /// Get the plugin metadata
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}