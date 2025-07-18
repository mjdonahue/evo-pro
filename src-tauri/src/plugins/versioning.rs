//! Plugin versioning and compatibility checking
//!
//! This module provides functionality for checking plugin version compatibility
//! with the host application.

use semver::{Version, VersionReq};
use crate::error::{Error, ErrorKind, Result};

use super::interfaces::PluginMetadata;

/// Host application version information
pub struct HostVersion {
    /// Current version of the host application
    pub current_version: Version,
    
    /// Minimum supported plugin API version
    pub min_plugin_api_version: Version,
    
    /// Maximum supported plugin API version
    pub max_plugin_api_version: Version,
}

impl HostVersion {
    /// Create a new host version
    pub fn new(current_version: Version, min_plugin_api_version: Version, max_plugin_api_version: Version) -> Self {
        Self {
            current_version,
            min_plugin_api_version,
            max_plugin_api_version,
        }
    }
    
    /// Create a host version from string versions
    pub fn from_strings(current_version: &str, min_plugin_api_version: &str, max_plugin_api_version: &str) -> Result<Self> {
        let current = Version::parse(current_version).map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to parse current version: {}", e)
            )
        })?;
        
        let min = Version::parse(min_plugin_api_version).map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to parse min plugin API version: {}", e)
            )
        })?;
        
        let max = Version::parse(max_plugin_api_version).map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to parse max plugin API version: {}", e)
            )
        })?;
        
        Ok(Self::new(current, min, max))
    }
}

/// Plugin version compatibility checker
pub struct VersionChecker {
    /// Host application version information
    host_version: HostVersion,
}

impl VersionChecker {
    /// Create a new version checker
    pub fn new(host_version: HostVersion) -> Self {
        Self {
            host_version,
        }
    }
    
    /// Check if a plugin is compatible with the host application
    pub fn check_compatibility(&self, metadata: &PluginMetadata) -> Result<()> {
        // Check if the plugin specifies a minimum host version
        if let Some(min_host_version) = &metadata.min_host_version {
            let req = VersionReq::parse(min_host_version).map_err(|e| {
                Error::new(
                    ErrorKind::Parse,
                    &format!("Failed to parse plugin min host version requirement: {}", e)
                )
            })?;
            
            if !req.matches(&self.host_version.current_version) {
                return Err(Error::new(
                    ErrorKind::Version,
                    &format!(
                        "Plugin {} requires host version {} but current version is {}",
                        metadata.id, min_host_version, self.host_version.current_version
                    )
                ));
            }
        }
        
        // Check if the plugin specifies a maximum host version
        if let Some(max_host_version) = &metadata.max_host_version {
            let req = VersionReq::parse(max_host_version).map_err(|e| {
                Error::new(
                    ErrorKind::Parse,
                    &format!("Failed to parse plugin max host version requirement: {}", e)
                )
            })?;
            
            if !req.matches(&self.host_version.current_version) {
                return Err(Error::new(
                    ErrorKind::Version,
                    &format!(
                        "Plugin {} requires host version <= {} but current version is {}",
                        metadata.id, max_host_version, self.host_version.current_version
                    )
                ));
            }
        }
        
        // Check if the plugin version is compatible with the host's supported plugin API versions
        let plugin_version = Version::parse(&metadata.version).map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to parse plugin version: {}", e)
            )
        })?;
        
        if plugin_version < self.host_version.min_plugin_api_version {
            return Err(Error::new(
                ErrorKind::Version,
                &format!(
                    "Plugin {} version {} is too old (minimum supported is {})",
                    metadata.id, plugin_version, self.host_version.min_plugin_api_version
                )
            ));
        }
        
        if plugin_version > self.host_version.max_plugin_api_version {
            return Err(Error::new(
                ErrorKind::Version,
                &format!(
                    "Plugin {} version {} is too new (maximum supported is {})",
                    metadata.id, plugin_version, self.host_version.max_plugin_api_version
                )
            ));
        }
        
        Ok(())
    }
    
    /// Get the host version information
    pub fn host_version(&self) -> &HostVersion {
        &self.host_version
    }
}

/// Version compatibility result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionCompatibility {
    /// Plugin is compatible with the host application
    Compatible,
    
    /// Plugin is too old for the host application
    PluginTooOld {
        /// Plugin version
        plugin_version: Version,
        
        /// Minimum supported plugin version
        min_supported: Version,
    },
    
    /// Plugin is too new for the host application
    PluginTooNew {
        /// Plugin version
        plugin_version: Version,
        
        /// Maximum supported plugin version
        max_supported: Version,
    },
    
    /// Host application is too old for the plugin
    HostTooOld {
        /// Host version
        host_version: Version,
        
        /// Minimum required host version
        min_required: Version,
    },
    
    /// Host application is too new for the plugin
    HostTooNew {
        /// Host version
        host_version: Version,
        
        /// Maximum required host version
        max_required: Version,
    },
}

impl VersionCompatibility {
    /// Check if the compatibility result is compatible
    pub fn is_compatible(&self) -> bool {
        matches!(self, VersionCompatibility::Compatible)
    }
    
    /// Get a human-readable error message for incompatible versions
    pub fn error_message(&self) -> Option<String> {
        match self {
            VersionCompatibility::Compatible => None,
            VersionCompatibility::PluginTooOld { plugin_version, min_supported } => {
                Some(format!(
                    "Plugin version {} is too old (minimum supported is {})",
                    plugin_version, min_supported
                ))
            },
            VersionCompatibility::PluginTooNew { plugin_version, max_supported } => {
                Some(format!(
                    "Plugin version {} is too new (maximum supported is {})",
                    plugin_version, max_supported
                ))
            },
            VersionCompatibility::HostTooOld { host_version, min_required } => {
                Some(format!(
                    "Host version {} is too old (minimum required is {})",
                    host_version, min_required
                ))
            },
            VersionCompatibility::HostTooNew { host_version, max_required } => {
                Some(format!(
                    "Host version {} is too new (maximum required is {})",
                    host_version, max_required
                ))
            },
        }
    }
}

/// Extended version checker with more detailed compatibility information
pub struct DetailedVersionChecker {
    /// Host application version information
    host_version: HostVersion,
}

impl DetailedVersionChecker {
    /// Create a new detailed version checker
    pub fn new(host_version: HostVersion) -> Self {
        Self {
            host_version,
        }
    }
    
    /// Check if a plugin is compatible with the host application
    pub fn check_compatibility(&self, metadata: &PluginMetadata) -> Result<VersionCompatibility> {
        // Parse the plugin version
        let plugin_version = Version::parse(&metadata.version).map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to parse plugin version: {}", e)
            )
        })?;
        
        // Check if the plugin version is compatible with the host's supported plugin API versions
        if plugin_version < self.host_version.min_plugin_api_version {
            return Ok(VersionCompatibility::PluginTooOld {
                plugin_version,
                min_supported: self.host_version.min_plugin_api_version.clone(),
            });
        }
        
        if plugin_version > self.host_version.max_plugin_api_version {
            return Ok(VersionCompatibility::PluginTooNew {
                plugin_version,
                max_supported: self.host_version.max_plugin_api_version.clone(),
            });
        }
        
        // Check if the plugin specifies a minimum host version
        if let Some(min_host_version) = &metadata.min_host_version {
            let min_version = Version::parse(min_host_version).map_err(|e| {
                Error::new(
                    ErrorKind::Parse,
                    &format!("Failed to parse plugin min host version: {}", e)
                )
            })?;
            
            if self.host_version.current_version < min_version {
                return Ok(VersionCompatibility::HostTooOld {
                    host_version: self.host_version.current_version.clone(),
                    min_required: min_version,
                });
            }
        }
        
        // Check if the plugin specifies a maximum host version
        if let Some(max_host_version) = &metadata.max_host_version {
            let max_version = Version::parse(max_host_version).map_err(|e| {
                Error::new(
                    ErrorKind::Parse,
                    &format!("Failed to parse plugin max host version: {}", e)
                )
            })?;
            
            if self.host_version.current_version > max_version {
                return Ok(VersionCompatibility::HostTooNew {
                    host_version: self.host_version.current_version.clone(),
                    max_required: max_version,
                });
            }
        }
        
        // If we get here, the plugin is compatible
        Ok(VersionCompatibility::Compatible)
    }
    
    /// Get the host version information
    pub fn host_version(&self) -> &HostVersion {
        &self.host_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::interfaces::{PluginType, PluginPermission};
    
    fn create_test_metadata(version: &str, min_host_version: Option<&str>, max_host_version: Option<&str>) -> PluginMetadata {
        PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: version.to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            permissions: vec![],
            signature: None,
            plugin_type: PluginType::BuiltIn,
            min_host_version: min_host_version.map(|v| v.to_string()),
            max_host_version: max_host_version.map(|v| v.to_string()),
            capabilities: vec![],
        }
    }
    
    #[test]
    fn test_version_checker_compatibility() {
        // Create a host version
        let host_version = HostVersion::from_strings("1.0.0", "0.5.0", "1.5.0").unwrap();
        
        // Create a version checker
        let checker = VersionChecker::new(host_version);
        
        // Test compatible plugin
        let metadata = create_test_metadata("1.0.0", None, None);
        assert!(checker.check_compatibility(&metadata).is_ok());
        
        // Test plugin that's too old
        let metadata = create_test_metadata("0.4.0", None, None);
        assert!(checker.check_compatibility(&metadata).is_err());
        
        // Test plugin that's too new
        let metadata = create_test_metadata("1.6.0", None, None);
        assert!(checker.check_compatibility(&metadata).is_err());
        
        // Test plugin with min host version that's compatible
        let metadata = create_test_metadata("1.0.0", Some("1.0.0"), None);
        assert!(checker.check_compatibility(&metadata).is_ok());
        
        // Test plugin with min host version that's incompatible
        let metadata = create_test_metadata("1.0.0", Some("2.0.0"), None);
        assert!(checker.check_compatibility(&metadata).is_err());
        
        // Test plugin with max host version that's compatible
        let metadata = create_test_metadata("1.0.0", None, Some("2.0.0"));
        assert!(checker.check_compatibility(&metadata).is_ok());
        
        // Test plugin with max host version that's incompatible
        let metadata = create_test_metadata("1.0.0", None, Some("0.9.0"));
        assert!(checker.check_compatibility(&metadata).is_err());
    }
    
    #[test]
    fn test_detailed_version_checker() {
        // Create a host version
        let host_version = HostVersion::from_strings("1.0.0", "0.5.0", "1.5.0").unwrap();
        
        // Create a detailed version checker
        let checker = DetailedVersionChecker::new(host_version);
        
        // Test compatible plugin
        let metadata = create_test_metadata("1.0.0", None, None);
        assert_eq!(checker.check_compatibility(&metadata).unwrap(), VersionCompatibility::Compatible);
        
        // Test plugin that's too old
        let metadata = create_test_metadata("0.4.0", None, None);
        match checker.check_compatibility(&metadata).unwrap() {
            VersionCompatibility::PluginTooOld { plugin_version, min_supported } => {
                assert_eq!(plugin_version, Version::parse("0.4.0").unwrap());
                assert_eq!(min_supported, Version::parse("0.5.0").unwrap());
            },
            other => panic!("Expected PluginTooOld, got {:?}", other),
        }
        
        // Test plugin that's too new
        let metadata = create_test_metadata("1.6.0", None, None);
        match checker.check_compatibility(&metadata).unwrap() {
            VersionCompatibility::PluginTooNew { plugin_version, max_supported } => {
                assert_eq!(plugin_version, Version::parse("1.6.0").unwrap());
                assert_eq!(max_supported, Version::parse("1.5.0").unwrap());
            },
            other => panic!("Expected PluginTooNew, got {:?}", other),
        }
        
        // Test plugin with min host version that's incompatible
        let metadata = create_test_metadata("1.0.0", Some("2.0.0"), None);
        match checker.check_compatibility(&metadata).unwrap() {
            VersionCompatibility::HostTooOld { host_version, min_required } => {
                assert_eq!(host_version, Version::parse("1.0.0").unwrap());
                assert_eq!(min_required, Version::parse("2.0.0").unwrap());
            },
            other => panic!("Expected HostTooOld, got {:?}", other),
        }
        
        // Test plugin with max host version that's incompatible
        let metadata = create_test_metadata("1.0.0", None, Some("0.9.0"));
        match checker.check_compatibility(&metadata).unwrap() {
            VersionCompatibility::HostTooNew { host_version, max_required } => {
                assert_eq!(host_version, Version::parse("1.0.0").unwrap());
                assert_eq!(max_required, Version::parse("0.9.0").unwrap());
            },
            other => panic!("Expected HostTooNew, got {:?}", other),
        }
    }
    
    #[test]
    fn test_version_compatibility_error_messages() {
        // Test error messages for each compatibility result
        let compatible = VersionCompatibility::Compatible;
        assert!(compatible.error_message().is_none());
        
        let plugin_too_old = VersionCompatibility::PluginTooOld {
            plugin_version: Version::parse("0.4.0").unwrap(),
            min_supported: Version::parse("0.5.0").unwrap(),
        };
        assert!(plugin_too_old.error_message().unwrap().contains("too old"));
        
        let plugin_too_new = VersionCompatibility::PluginTooNew {
            plugin_version: Version::parse("1.6.0").unwrap(),
            max_supported: Version::parse("1.5.0").unwrap(),
        };
        assert!(plugin_too_new.error_message().unwrap().contains("too new"));
        
        let host_too_old = VersionCompatibility::HostTooOld {
            host_version: Version::parse("1.0.0").unwrap(),
            min_required: Version::parse("2.0.0").unwrap(),
        };
        assert!(host_too_old.error_message().unwrap().contains("too old"));
        
        let host_too_new = VersionCompatibility::HostTooNew {
            host_version: Version::parse("1.0.0").unwrap(),
            max_required: Version::parse("0.9.0").unwrap(),
        };
        assert!(host_too_new.error_message().unwrap().contains("too new"));
    }
}