//! Plugin system for extending application functionality
//! 
//! This module provides a secure plugin architecture that allows for
//! extending the application with custom functionality while maintaining
//! security and stability.

mod interfaces;
mod manager;
mod loader;
mod security;
mod registry;
mod capabilities;
mod versioning;
mod marketplace;

pub use interfaces::*;
pub use manager::*;
pub use loader::*;
pub use security::*;
pub use registry::*;
pub use capabilities::*;
pub use versioning::*;
pub use marketplace::*;

use std::sync::{Arc, Mutex, Once};
use crate::error::Result;

/// Global plugin manager instance
static PLUGIN_MANAGER_INIT: Once = Once::new();
static mut PLUGIN_MANAGER: Option<Arc<Mutex<PluginManager>>> = None;

/// Get the global plugin manager instance
pub fn get_plugin_manager() -> Arc<Mutex<PluginManager>> {
    unsafe {
        PLUGIN_MANAGER_INIT.call_once(|| {
            // Create a new plugin manager
            let plugin_manager = PluginManager::new();

            PLUGIN_MANAGER = Some(Arc::new(Mutex::new(plugin_manager)));
        });

        PLUGIN_MANAGER.clone().unwrap()
    }
}

/// Initialize the plugin system
pub fn init() -> Result<()> {
    tracing::info!("Initializing plugin system");

    // Initialize the plugin manager
    let plugin_manager = get_plugin_manager();

    // Load built-in plugins
    {
        let mut manager = plugin_manager.lock().unwrap();
        manager.load_built_in_plugins()?;
    }

    // Discover and load external plugins
    {
        let mut manager = plugin_manager.lock().unwrap();
        manager.discover_plugins()?;
    }

    // Initialize the marketplace manager
    let marketplace_manager = marketplace::get_marketplace_manager();

    // Perform initial marketplace refresh in the background
    tokio::spawn(async move {
        let mut manager = marketplace_manager.lock().unwrap();
        if let Err(e) = manager.refresh().await {
            tracing::warn!("Failed to perform initial marketplace refresh: {}", e);
        } else {
            tracing::info!("Initial marketplace refresh completed");
        }
    });

    tracing::info!("Plugin system initialized");
    Ok(())
}
