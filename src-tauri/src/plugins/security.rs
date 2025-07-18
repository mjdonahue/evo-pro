//! Security measures for plugins
//!
//! This module provides security measures for plugins, including
//! sandboxing, signature validation, and permission enforcement.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use async_trait::async_trait;
use crate::error::{Error, ErrorKind, Result};

use super::interfaces::{Plugin, PluginMetadata, PluginPermission};

/// Plugin security validator
///
/// This trait defines the interface for validating plugin security.
#[async_trait]
pub trait PluginSecurityValidator: Send + Sync {
    /// Validate a plugin's security
    async fn validate_plugin(&self, metadata: &PluginMetadata, path: Option<&Path>) -> Result<()>;
    
    /// Check if a plugin has a specific permission
    fn has_permission(&self, plugin_id: &str, permission: &PluginPermission) -> bool;
    
    /// Grant a permission to a plugin
    fn grant_permission(&mut self, plugin_id: &str, permission: PluginPermission) -> Result<()>;
    
    /// Revoke a permission from a plugin
    fn revoke_permission(&mut self, plugin_id: &str, permission: &PluginPermission) -> Result<()>;
}

/// Default plugin security validator
pub struct DefaultPluginSecurityValidator {
    /// Permissions granted to plugins
    permissions: HashMap<String, HashSet<PluginPermission>>,
    
    /// Trusted plugin signatures
    trusted_signatures: HashSet<String>,
    
    /// Whether to require signatures for all plugins
    require_signatures: bool,
}

impl DefaultPluginSecurityValidator {
    /// Create a new default plugin security validator
    pub fn new(require_signatures: bool) -> Self {
        Self {
            permissions: HashMap::new(),
            trusted_signatures: HashSet::new(),
            require_signatures,
        }
    }
    
    /// Add a trusted signature
    pub fn add_trusted_signature(&mut self, signature: String) {
        self.trusted_signatures.insert(signature);
    }
    
    /// Remove a trusted signature
    pub fn remove_trusted_signature(&mut self, signature: &str) -> bool {
        self.trusted_signatures.remove(signature)
    }
    
    /// Set whether to require signatures for all plugins
    pub fn set_require_signatures(&mut self, require: bool) {
        self.require_signatures = require;
    }
    
    /// Get whether signatures are required for all plugins
    pub fn require_signatures(&self) -> bool {
        self.require_signatures
    }
    
    /// Validate a plugin's signature
    fn validate_signature(&self, metadata: &PluginMetadata) -> Result<()> {
        // If signatures are not required, skip validation
        if !self.require_signatures {
            return Ok(());
        }
        
        // Check if the plugin has a signature
        let signature = metadata.signature.as_ref().ok_or_else(|| {
            Error::new(
                ErrorKind::Security,
                &format!("Plugin {} does not have a signature", metadata.id)
            )
        })?;
        
        // Check if the signature is trusted
        if !self.trusted_signatures.contains(signature) {
            return Err(Error::new(
                ErrorKind::Security,
                &format!("Plugin {} has an untrusted signature", metadata.id)
            ));
        }
        
        Ok(())
    }
    
    /// Validate a plugin's requested permissions
    fn validate_permissions(&self, metadata: &PluginMetadata) -> Result<()> {
        // Check if the plugin requests any dangerous permissions
        for permission in &metadata.permissions {
            match permission {
                PluginPermission::FileSystem { .. } => {
                    // File system access is potentially dangerous
                    // In a real implementation, we would check if the plugin is trusted
                    // or if the user has explicitly granted this permission
                    return Err(Error::new(
                        ErrorKind::Security,
                        &format!("Plugin {} requests dangerous file system permission", metadata.id)
                    ));
                },
                PluginPermission::Network { .. } => {
                    // Network access is potentially dangerous
                    // In a real implementation, we would check if the plugin is trusted
                    // or if the user has explicitly granted this permission
                    return Err(Error::new(
                        ErrorKind::Security,
                        &format!("Plugin {} requests dangerous network permission", metadata.id)
                    ));
                },
                PluginPermission::Process { .. } => {
                    // Process execution is potentially dangerous
                    // In a real implementation, we would check if the plugin is trusted
                    // or if the user has explicitly granted this permission
                    return Err(Error::new(
                        ErrorKind::Security,
                        &format!("Plugin {} requests dangerous process permission", metadata.id)
                    ));
                },
                // Other permissions are considered safe by default
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Get all permissions granted to a plugin
    pub fn get_permissions(&self, plugin_id: &str) -> HashSet<PluginPermission> {
        self.permissions.get(plugin_id).cloned().unwrap_or_default()
    }
}

#[async_trait]
impl PluginSecurityValidator for DefaultPluginSecurityValidator {
    async fn validate_plugin(&self, metadata: &PluginMetadata, _path: Option<&Path>) -> Result<()> {
        // Validate the plugin's signature
        self.validate_signature(metadata)?;
        
        // Validate the plugin's requested permissions
        self.validate_permissions(metadata)?;
        
        Ok(())
    }
    
    fn has_permission(&self, plugin_id: &str, permission: &PluginPermission) -> bool {
        if let Some(permissions) = self.permissions.get(plugin_id) {
            permissions.contains(permission)
        } else {
            false
        }
    }
    
    fn grant_permission(&mut self, plugin_id: &str, permission: PluginPermission) -> Result<()> {
        // Get or create the permission set for this plugin
        let permissions = self.permissions.entry(plugin_id.to_string()).or_insert_with(HashSet::new);
        
        // Add the permission
        permissions.insert(permission);
        
        Ok(())
    }
    
    fn revoke_permission(&mut self, plugin_id: &str, permission: &PluginPermission) -> Result<()> {
        // Get the permission set for this plugin
        if let Some(permissions) = self.permissions.get_mut(plugin_id) {
            // Remove the permission
            permissions.remove(permission);
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                &format!("No permissions found for plugin: {}", plugin_id)
            ))
        }
    }
}

impl Default for DefaultPluginSecurityValidator {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Plugin sandbox
///
/// This trait defines the interface for sandboxing plugins.
#[async_trait]
pub trait PluginSandbox: Send + Sync {
    /// Create a sandbox for a plugin
    async fn create_sandbox(&self, metadata: &PluginMetadata) -> Result<Box<dyn PluginSandboxInstance>>;
}

/// Plugin sandbox instance
///
/// This trait defines the interface for a plugin sandbox instance.
#[async_trait]
pub trait PluginSandboxInstance: Send + Sync {
    /// Execute a function in the sandbox
    async fn execute<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send,
        R: Send;
    
    /// Destroy the sandbox
    async fn destroy(&self) -> Result<()>;
}

/// No-op plugin sandbox
///
/// This sandbox doesn't provide any actual sandboxing, it just executes
/// the function directly. This is useful for built-in plugins that are
/// trusted and don't need sandboxing.
pub struct NoOpPluginSandbox;

impl NoOpPluginSandbox {
    /// Create a new no-op plugin sandbox
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PluginSandbox for NoOpPluginSandbox {
    async fn create_sandbox(&self, _metadata: &PluginMetadata) -> Result<Box<dyn PluginSandboxInstance>> {
        Ok(Box::new(NoOpPluginSandboxInstance))
    }
}

/// No-op plugin sandbox instance
pub struct NoOpPluginSandboxInstance;

#[async_trait]
impl PluginSandboxInstance for NoOpPluginSandboxInstance {
    async fn execute<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send,
        R: Send,
    {
        f()
    }
    
    async fn destroy(&self) -> Result<()> {
        Ok(())
    }
}

/// Plugin security manager
///
/// This struct manages plugin security, including validation and sandboxing.
pub struct PluginSecurityManager {
    /// Plugin security validator
    validator: Box<dyn PluginSecurityValidator>,
    
    /// Plugin sandbox
    sandbox: Box<dyn PluginSandbox>,
}

impl PluginSecurityManager {
    /// Create a new plugin security manager
    pub fn new(validator: Box<dyn PluginSecurityValidator>, sandbox: Box<dyn PluginSandbox>) -> Self {
        Self {
            validator,
            sandbox,
        }
    }
    
    /// Create a default plugin security manager
    pub fn default_manager() -> Self {
        Self {
            validator: Box::new(DefaultPluginSecurityValidator::default()),
            sandbox: Box::new(NoOpPluginSandbox::new()),
        }
    }
    
    /// Validate a plugin's security
    pub async fn validate_plugin(&self, metadata: &PluginMetadata, path: Option<&Path>) -> Result<()> {
        self.validator.validate_plugin(metadata, path).await
    }
    
    /// Check if a plugin has a specific permission
    pub fn has_permission(&self, plugin_id: &str, permission: &PluginPermission) -> bool {
        self.validator.has_permission(plugin_id, permission)
    }
    
    /// Grant a permission to a plugin
    pub fn grant_permission(&mut self, plugin_id: &str, permission: PluginPermission) -> Result<()> {
        self.validator.grant_permission(plugin_id, permission)
    }
    
    /// Revoke a permission from a plugin
    pub fn revoke_permission(&mut self, plugin_id: &str, permission: &PluginPermission) -> Result<()> {
        self.validator.revoke_permission(plugin_id, permission)
    }
    
    /// Create a sandbox for a plugin
    pub async fn create_sandbox(&self, metadata: &PluginMetadata) -> Result<Box<dyn PluginSandboxInstance>> {
        self.sandbox.create_sandbox(metadata).await
    }
    
    /// Get the plugin security validator
    pub fn validator(&self) -> &dyn PluginSecurityValidator {
        self.validator.as_ref()
    }
    
    /// Get the plugin sandbox
    pub fn sandbox(&self) -> &dyn PluginSandbox {
        self.sandbox.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_default_security_validator() {
        // Create a validator that doesn't require signatures
        let mut validator = DefaultPluginSecurityValidator::new(false);
        
        // Create a plugin metadata with no signature
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            signature: None,
            plugin_type: super::super::interfaces::PluginType::BuiltIn,
            min_host_version: None,
            max_host_version: None,
            capabilities: vec![],
        };
        
        // Validate the plugin (should succeed because signatures are not required)
        assert!(validator.validate_plugin(&metadata, None).await.is_ok());
        
        // Now require signatures
        validator.set_require_signatures(true);
        
        // Validate the plugin again (should fail because it has no signature)
        assert!(validator.validate_plugin(&metadata, None).await.is_err());
        
        // Add a signature to the plugin
        let mut metadata_with_sig = metadata.clone();
        metadata_with_sig.signature = Some("test-signature".to_string());
        
        // Validate the plugin again (should still fail because the signature is not trusted)
        assert!(validator.validate_plugin(&metadata_with_sig, None).await.is_err());
        
        // Add the signature to the trusted signatures
        validator.add_trusted_signature("test-signature".to_string());
        
        // Validate the plugin again (should succeed now)
        assert!(validator.validate_plugin(&metadata_with_sig, None).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_permission_management() {
        // Create a validator
        let mut validator = DefaultPluginSecurityValidator::new(false);
        
        // Define a plugin ID and some permissions
        let plugin_id = "test-plugin";
        let permission1 = PluginPermission::UI { component: "test-component".to_string() };
        let permission2 = PluginPermission::Storage { key_prefix: "test-prefix".to_string() };
        
        // Initially, the plugin should have no permissions
        assert!(!validator.has_permission(plugin_id, &permission1));
        assert!(!validator.has_permission(plugin_id, &permission2));
        
        // Grant a permission
        assert!(validator.grant_permission(plugin_id, permission1.clone()).is_ok());
        
        // Now the plugin should have the first permission but not the second
        assert!(validator.has_permission(plugin_id, &permission1));
        assert!(!validator.has_permission(plugin_id, &permission2));
        
        // Grant the second permission
        assert!(validator.grant_permission(plugin_id, permission2.clone()).is_ok());
        
        // Now the plugin should have both permissions
        assert!(validator.has_permission(plugin_id, &permission1));
        assert!(validator.has_permission(plugin_id, &permission2));
        
        // Revoke the first permission
        assert!(validator.revoke_permission(plugin_id, &permission1).is_ok());
        
        // Now the plugin should have only the second permission
        assert!(!validator.has_permission(plugin_id, &permission1));
        assert!(validator.has_permission(plugin_id, &permission2));
        
        // Revoking a permission from a non-existent plugin should fail
        assert!(validator.revoke_permission("non-existent", &permission1).is_err());
    }
    
    #[tokio::test]
    async fn test_dangerous_permissions() {
        // Create a validator
        let validator = DefaultPluginSecurityValidator::new(false);
        
        // Create a plugin metadata with dangerous permissions
        let metadata_with_fs = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![
                PluginPermission::FileSystem { path: "/tmp".to_string() }
            ],
            signature: None,
            plugin_type: super::super::interfaces::PluginType::BuiltIn,
            min_host_version: None,
            max_host_version: None,
            capabilities: vec![],
        };
        
        // Validate the plugin (should fail because it has dangerous permissions)
        assert!(validator.validate_plugin(&metadata_with_fs, None).await.is_err());
        
        // Create a plugin metadata with network permissions
        let metadata_with_network = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![
                PluginPermission::Network { host: "example.com".to_string() }
            ],
            signature: None,
            plugin_type: super::super::interfaces::PluginType::BuiltIn,
            min_host_version: None,
            max_host_version: None,
            capabilities: vec![],
        };
        
        // Validate the plugin (should fail because it has dangerous permissions)
        assert!(validator.validate_plugin(&metadata_with_network, None).await.is_err());
    }
    
    #[tokio::test]
    async fn test_noop_sandbox() {
        // Create a sandbox
        let sandbox = NoOpPluginSandbox::new();
        
        // Create a plugin metadata
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            signature: None,
            plugin_type: super::super::interfaces::PluginType::BuiltIn,
            min_host_version: None,
            max_host_version: None,
            capabilities: vec![],
        };
        
        // Create a sandbox instance
        let sandbox_instance = sandbox.create_sandbox(&metadata).await.unwrap();
        
        // Execute a function in the sandbox
        let result = sandbox_instance.execute(|| Ok(42)).await.unwrap();
        assert_eq!(result, 42);
        
        // Destroy the sandbox
        assert!(sandbox_instance.destroy().await.is_ok());
    }
}