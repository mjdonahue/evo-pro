//! Plugin capability negotiation
//!
//! This module provides functionality for plugin capability negotiation,
//! allowing plugins to declare what capabilities they need and for the
//! host application to check if those capabilities are available.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::error::{Error, ErrorKind, Result};

/// Plugin capability
///
/// A capability represents a feature or functionality that a plugin
/// can use or provide.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    
    /// Capability version
    pub version: String,
    
    /// Capability parameters
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

impl Capability {
    /// Create a new capability
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            parameters: HashMap::new(),
        }
    }
    
    /// Create a new capability with parameters
    pub fn with_parameters(name: &str, version: &str, parameters: HashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            parameters,
        }
    }
    
    /// Add a parameter to the capability
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if this capability is compatible with another capability
    pub fn is_compatible_with(&self, other: &Capability) -> bool {
        // Check if the names match
        if self.name != other.name {
            return false;
        }
        
        // Check if the versions are compatible
        if !self.is_version_compatible_with(&other.version) {
            return false;
        }
        
        // Check if all required parameters are present and compatible
        for (key, value) in &self.parameters {
            if let Some(other_value) = other.parameters.get(key) {
                if value != other_value {
                    return false;
                }
            } else {
                // If the parameter is not present in the other capability,
                // it's not compatible
                return false;
            }
        }
        
        true
    }
    
    /// Check if this capability's version is compatible with another version
    fn is_version_compatible_with(&self, other_version: &str) -> bool {
        // Parse the versions
        let self_version = semver::Version::parse(&self.version).ok();
        let other_version = semver::Version::parse(other_version).ok();
        
        match (self_version, other_version) {
            (Some(self_version), Some(other_version)) => {
                // Check if the major versions match
                if self_version.major != other_version.major {
                    return false;
                }
                
                // Check if the other version is greater than or equal to this version
                other_version >= self_version
            },
            _ => {
                // If we can't parse the versions, fall back to string comparison
                self.version == other_version
            }
        }
    }
}

/// Capability registry
///
/// The capability registry keeps track of all available capabilities
/// in the host application.
pub struct CapabilityRegistry {
    /// Available capabilities
    capabilities: HashSet<Capability>,
}

impl CapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }
    
    /// Register a capability
    pub fn register_capability(&mut self, capability: Capability) -> Result<()> {
        // Check if the capability is already registered
        if self.capabilities.contains(&capability) {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                &format!("Capability {} is already registered", capability.name)
            ));
        }
        
        // Register the capability
        self.capabilities.insert(capability);
        
        Ok(())
    }
    
    /// Unregister a capability
    pub fn unregister_capability(&mut self, capability: &Capability) -> Result<()> {
        // Check if the capability is registered
        if !self.capabilities.contains(capability) {
            return Err(Error::new(
                ErrorKind::NotFound,
                &format!("Capability {} is not registered", capability.name)
            ));
        }
        
        // Unregister the capability
        self.capabilities.remove(capability);
        
        Ok(())
    }
    
    /// Check if a capability is available
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.iter().any(|c| c.is_compatible_with(capability))
    }
    
    /// Get all available capabilities
    pub fn get_capabilities(&self) -> &HashSet<Capability> {
        &self.capabilities
    }
    
    /// Find a capability by name
    pub fn find_capability(&self, name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.name == name)
    }
    
    /// Find all capabilities that match a name
    pub fn find_capabilities(&self, name: &str) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.name == name).collect()
    }
    
    /// Clear all registered capabilities
    pub fn clear(&mut self) {
        self.capabilities.clear();
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability negotiator
///
/// The capability negotiator is responsible for negotiating capabilities
/// between plugins and the host application.
pub struct CapabilityNegotiator {
    /// Capability registry
    registry: CapabilityRegistry,
}

impl CapabilityNegotiator {
    /// Create a new capability negotiator
    pub fn new(registry: CapabilityRegistry) -> Self {
        Self {
            registry,
        }
    }
    
    /// Negotiate capabilities for a plugin
    pub fn negotiate_capabilities(&self, required_capabilities: &[Capability]) -> Result<()> {
        // Check if all required capabilities are available
        for capability in required_capabilities {
            if !self.registry.has_capability(capability) {
                return Err(Error::new(
                    ErrorKind::Capability,
                    &format!("Required capability {} is not available", capability.name)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Get the capability registry
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }
    
    /// Get a mutable reference to the capability registry
    pub fn registry_mut(&mut self) -> &mut CapabilityRegistry {
        &mut self.registry
    }
}

impl Default for CapabilityNegotiator {
    fn default() -> Self {
        Self::new(CapabilityRegistry::new())
    }
}

/// Standard capabilities
///
/// This module defines standard capabilities that are available in the
/// host application.
pub mod standard {
    use super::*;
    
    /// UI capability
    pub fn ui(version: &str) -> Capability {
        Capability::new("ui", version)
    }
    
    /// Storage capability
    pub fn storage(version: &str) -> Capability {
        Capability::new("storage", version)
    }
    
    /// Network capability
    pub fn network(version: &str) -> Capability {
        Capability::new("network", version)
    }
    
    /// File system capability
    pub fn filesystem(version: &str) -> Capability {
        Capability::new("filesystem", version)
    }
    
    /// Process capability
    pub fn process(version: &str) -> Capability {
        Capability::new("process", version)
    }
    
    /// API capability
    pub fn api(version: &str, api_name: &str) -> Capability {
        Capability::new("api", version)
            .with_parameter("name", api_name)
    }
    
    /// Event capability
    pub fn event(version: &str, event_type: &str) -> Capability {
        Capability::new("event", version)
            .with_parameter("type", event_type)
    }
    
    /// Register standard capabilities
    pub fn register_standard_capabilities(registry: &mut CapabilityRegistry) -> Result<()> {
        // Register UI capability
        registry.register_capability(ui("1.0.0"))?;
        
        // Register storage capability
        registry.register_capability(storage("1.0.0"))?;
        
        // Register network capability
        registry.register_capability(network("1.0.0"))?;
        
        // Register file system capability
        registry.register_capability(filesystem("1.0.0"))?;
        
        // Register process capability
        registry.register_capability(process("1.0.0"))?;
        
        // Register API capabilities
        registry.register_capability(api("1.0.0", "core"))?;
        registry.register_capability(api("1.0.0", "ui"))?;
        registry.register_capability(api("1.0.0", "storage"))?;
        
        // Register event capabilities
        registry.register_capability(event("1.0.0", "system"))?;
        registry.register_capability(event("1.0.0", "ui"))?;
        registry.register_capability(event("1.0.0", "storage"))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_compatibility() {
        // Create capabilities
        let cap1 = Capability::new("test", "1.0.0");
        let cap2 = Capability::new("test", "1.0.0");
        let cap3 = Capability::new("test", "1.1.0");
        let cap4 = Capability::new("test", "2.0.0");
        let cap5 = Capability::new("other", "1.0.0");
        
        // Test compatibility
        assert!(cap1.is_compatible_with(&cap2)); // Same version
        assert!(cap1.is_compatible_with(&cap3)); // Compatible version (1.0.0 is compatible with 1.1.0)
        assert!(!cap1.is_compatible_with(&cap4)); // Incompatible version (1.0.0 is not compatible with 2.0.0)
        assert!(!cap1.is_compatible_with(&cap5)); // Different name
    }
    
    #[test]
    fn test_capability_parameters() {
        // Create capabilities with parameters
        let cap1 = Capability::new("test", "1.0.0")
            .with_parameter("param1", "value1")
            .with_parameter("param2", "value2");
        
        let cap2 = Capability::new("test", "1.0.0")
            .with_parameter("param1", "value1")
            .with_parameter("param2", "value2");
        
        let cap3 = Capability::new("test", "1.0.0")
            .with_parameter("param1", "value1")
            .with_parameter("param2", "different");
        
        let cap4 = Capability::new("test", "1.0.0")
            .with_parameter("param1", "value1");
        
        // Test compatibility
        assert!(cap1.is_compatible_with(&cap2)); // Same parameters
        assert!(!cap1.is_compatible_with(&cap3)); // Different parameter value
        assert!(!cap1.is_compatible_with(&cap4)); // Missing parameter
        assert!(cap4.is_compatible_with(&cap1)); // Extra parameters are ok
    }
    
    #[test]
    fn test_capability_registry() {
        // Create a registry
        let mut registry = CapabilityRegistry::new();
        
        // Create capabilities
        let cap1 = Capability::new("test1", "1.0.0");
        let cap2 = Capability::new("test2", "1.0.0");
        
        // Register capabilities
        assert!(registry.register_capability(cap1.clone()).is_ok());
        assert!(registry.register_capability(cap2.clone()).is_ok());
        
        // Check if capabilities are registered
        assert!(registry.has_capability(&cap1));
        assert!(registry.has_capability(&cap2));
        
        // Try to register the same capability again
        assert!(registry.register_capability(cap1.clone()).is_err());
        
        // Unregister a capability
        assert!(registry.unregister_capability(&cap1).is_ok());
        
        // Check if the capability is unregistered
        assert!(!registry.has_capability(&cap1));
        assert!(registry.has_capability(&cap2));
        
        // Try to unregister a non-existent capability
        assert!(registry.unregister_capability(&cap1).is_err());
    }
    
    #[test]
    fn test_capability_negotiation() {
        // Create a registry
        let mut registry = CapabilityRegistry::new();
        
        // Register standard capabilities
        assert!(standard::register_standard_capabilities(&mut registry).is_ok());
        
        // Create a negotiator
        let negotiator = CapabilityNegotiator::new(registry);
        
        // Create required capabilities
        let required_caps = vec![
            standard::ui("1.0.0"),
            standard::storage("1.0.0"),
            standard::api("1.0.0", "core"),
        ];
        
        // Negotiate capabilities (should succeed)
        assert!(negotiator.negotiate_capabilities(&required_caps).is_ok());
        
        // Create required capabilities with a missing capability
        let required_caps = vec![
            standard::ui("1.0.0"),
            standard::storage("1.0.0"),
            Capability::new("missing", "1.0.0"),
        ];
        
        // Negotiate capabilities (should fail)
        assert!(negotiator.negotiate_capabilities(&required_caps).is_err());
    }
    
    #[test]
    fn test_version_compatibility() {
        // Create capabilities with different versions
        let cap1 = Capability::new("test", "1.0.0");
        let cap2 = Capability::new("test", "1.1.0");
        let cap3 = Capability::new("test", "1.2.0");
        let cap4 = Capability::new("test", "2.0.0");
        
        // Test compatibility
        assert!(cap1.is_compatible_with(&cap1)); // Same version
        assert!(cap1.is_compatible_with(&cap2)); // 1.0.0 is compatible with 1.1.0
        assert!(cap1.is_compatible_with(&cap3)); // 1.0.0 is compatible with 1.2.0
        assert!(!cap1.is_compatible_with(&cap4)); // 1.0.0 is not compatible with 2.0.0
        
        assert!(!cap2.is_compatible_with(&cap1)); // 1.1.0 is not compatible with 1.0.0 (older version)
        assert!(cap2.is_compatible_with(&cap2)); // Same version
        assert!(cap2.is_compatible_with(&cap3)); // 1.1.0 is compatible with 1.2.0
        assert!(!cap2.is_compatible_with(&cap4)); // 1.1.0 is not compatible with 2.0.0
        
        assert!(!cap3.is_compatible_with(&cap1)); // 1.2.0 is not compatible with 1.0.0 (older version)
        assert!(!cap3.is_compatible_with(&cap2)); // 1.2.0 is not compatible with 1.1.0 (older version)
        assert!(cap3.is_compatible_with(&cap3)); // Same version
        assert!(!cap3.is_compatible_with(&cap4)); // 1.2.0 is not compatible with 2.0.0
        
        assert!(!cap4.is_compatible_with(&cap1)); // 2.0.0 is not compatible with 1.0.0
        assert!(!cap4.is_compatible_with(&cap2)); // 2.0.0 is not compatible with 1.1.0
        assert!(!cap4.is_compatible_with(&cap3)); // 2.0.0 is not compatible with 1.2.0
        assert!(cap4.is_compatible_with(&cap4)); // Same version
    }
}