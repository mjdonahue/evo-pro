//! Plugin marketplace service for discovering and installing plugins
//!
//! This module provides Tauri commands for interacting with the plugin marketplace.

use crate::error::Result;
use crate::plugins::marketplace::{
    MarketplaceSource, MarketplaceEntry, get_marketplace_sources, get_marketplace_entries,
    search_marketplace_entries, install_plugin, uninstall_plugin, update_plugin, refresh_marketplace
};

/// Get all marketplace sources
#[tauri::command]
pub async fn get_plugin_marketplace_sources() -> Result<Vec<MarketplaceSource>> {
    get_marketplace_sources().await
}

/// Get all marketplace entries
#[tauri::command]
pub async fn get_plugin_marketplace_entries() -> Result<Vec<MarketplaceEntry>> {
    get_marketplace_entries().await
}

/// Search marketplace entries
#[tauri::command]
pub async fn search_plugin_marketplace(query: String) -> Result<Vec<MarketplaceEntry>> {
    search_marketplace_entries(query).await
}

/// Install a plugin from the marketplace
#[tauri::command]
pub async fn install_plugin_from_marketplace(plugin_id: String) -> Result<()> {
    install_plugin(plugin_id).await
}

/// Uninstall a plugin
#[tauri::command]
pub async fn uninstall_plugin_from_marketplace(plugin_id: String) -> Result<()> {
    uninstall_plugin(plugin_id).await
}

/// Update a plugin
#[tauri::command]
pub async fn update_plugin_from_marketplace(plugin_id: String) -> Result<()> {
    update_plugin(plugin_id).await
}

/// Refresh marketplace entries
#[tauri::command]
pub async fn refresh_plugin_marketplace() -> Result<()> {
    refresh_marketplace().await
}