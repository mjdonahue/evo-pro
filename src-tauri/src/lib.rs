pub mod actors;
pub mod commands;
pub mod entities;
pub mod error;
pub mod integration;
pub mod keys;
pub mod logging;
pub mod plugins;
pub mod privacy;
pub mod repositories;
pub mod resources;
pub mod security;
pub mod state;
pub mod storage;
// pub mod swarms;
pub mod utils;
use sqlx::migrate::MigrateError;
use tauri::Manager;
use tracing_subscriber::{EnvFilter, prelude::*};
use url::Url;

use crate::{
    actors::setup_actors, resources, security::secure_defaults::DatabaseSecureDefaults,
    services::security::SecurityService, state::AppState, storage::db::DatabaseManager, utils::get_data_dir,
};
#[tokio::main]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    dotenvy::dotenv().ok();
    color_eyre::install().unwrap();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true),
        )
        .init();

    // Initialize database and services
    let db_path = get_data_dir().join("data.db");
    let url = Url::from_file_path(&db_path).unwrap();

    // Use secure connection string for database
    let secure_connection_string = DatabaseSecureDefaults::get_connection_string(&url.to_string());
    let mut db = DatabaseManager::new(&secure_connection_string)
        .await
        .expect("Failed to initialize database");

    // Handle migration with version mismatch recovery
    if let Err(e) = db.run_migrations().await {
        match e {
            MigrateError::VersionMismatch(_) => {
                tracing::warn!("Migration version mismatch detected, recreating database...");

                // Close the database connection
                db.pool.close().await;

                // Delete the database file
                if db_path.exists() {
                    std::fs::remove_file(&db_path).expect("Failed to delete database file");
                }

                // Recreate the database connection
                db = DatabaseManager::new(&url.to_string())
                    .await
                    .expect("Failed to reinitialize database");

                // Rerun migrations on the fresh database
                db.run_migrations()
                    .await
                    .expect("Failed to run migrations after reset");
            }
            _ => panic!("Migration failed: {}", e),
        }
    }

    // Initialize resource detection and adaptation
    tracing::info!("Initializing resource detection and adaptation...");
    resources::initialize();

    // Start background resource monitoring
    let resource_manager = resources::adaptation::ResourceManager::global();
    let _monitor_handle = resources::adaptation::ResourceManager::start_background_adaptation(
        resource_manager.clone(),
        std::time::Duration::from_secs(60),
    );

    // Initialize plugin system
    tracing::info!("Initializing plugin system...");
    plugins::init().expect("Failed to initialize plugin system");

    // Initialize security service and apply secure defaults
    tracing::info!("Initializing security service and applying secure defaults...");
    let security_service = SecurityService::new(db.pool.clone())
        .await
        .expect("Failed to initialize security service");

    // Apply secure defaults to all components
    security_service.apply_secure_defaults()
        .await
        .expect("Failed to apply secure defaults");

    // Initialize default threat models
    security_service.initialize_default_models()
        .await
        .expect("Failed to initialize default threat models");

    // Note: This is a placeholder AppState creation - the actual actors need to be properly initialized

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            tokio::spawn(async move {
                handle.clone().manage(AppState {
                    app: handle.clone(),
                    actors: setup_actors(handle, db.clone())
                        .await
                        .expect("Failed to initialize actors"),
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::log_frontend_message,
            // Legacy agent command
            commands::create_task,
            commands::delete_tasks,
            commands::list_tasks,
            commands::update_task,
            commands::create_user,
            commands::list_users,
            commands::update_user,
            commands::delete_user,
            commands::create_conversation,
            commands::list_conversations,
            commands::create_agent,
            commands::update_agent,
            commands::list_agents,
            commands::get_public_key,
            commands::create_p2p_node,
            commands::update_p2p_node,
            commands::delete_p2p_node,
            commands::create_participant,
            commands::update_participant,
            commands::delete_participant,
            commands::list_participants,
            // Data management commands
            services::export_user_data,
            services::get_retention_policy,
            services::set_retention_policy,
            services::apply_retention_policy,
            // Data usage reporting commands
            services::generate_data_usage_report,
            services::update_data_preferences,
            // User consent management commands
            services::get_user_consent,
            services::update_user_consent,
            // Data deletion verification commands
            services::verify_data_deletion,
            services::generate_deletion_certificate,
            // Plugin marketplace commands
            services::get_plugin_marketplace_sources,
            services::get_plugin_marketplace_entries,
            services::search_plugin_marketplace,
            services::install_plugin_from_marketplace,
            services::uninstall_plugin_from_marketplace,
            services::update_plugin_from_marketplace,
            services::refresh_plugin_marketplace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use color_eyre::eyre::eyre;
    use rig::{
        client::CompletionClient, client::ProviderClient, completion::Prompt, providers::openai,
    };
    use tracing::info;

    #[tokio::test]
    async fn test_rig_integration() -> Result<()> {
        println!("Running rig integration test...");

        // Set up a dummy API key for testing (will fail gracefully if not set)
        if std::env::var("OPENAI_API_KEY").is_err() {
            println!("Skipping test - OPENAI_API_KEY not set");
            return Ok(());
        }

        // Create OpenAI client and model using rig-core
        let openai_client = openai::Client::from_env();
        let gpt_model = openai_client
            .agent("gpt-3.5-turbo")
            .preamble("You are an AI agent with a specialty in programming.")
            .build();

        // Send a prompt to the model
        let response = gpt_model
            .prompt("Hello! How are you? Please write a generic binary search function in Rust.")
            .await
            .map_err(|e| eyre!("Failed to get response from model: {}", e))?;

        info!("Model Response: {}", response);

        // Basic validation that we got a response
        assert!(!response.is_empty(), "Response should not be empty");
        assert!(
            response.to_lowercase().contains("rust") || response.contains("fn"),
            "Response should contain Rust-related content"
        );

        println!("✅ Rig integration test passed!");
        Ok(())
    }

    #[tokio::test]
    async fn test_agent_without_api() -> Result<()> {
        println!("Running agent creation test (no API call)...");

        // Test that we can create agents without making API calls
        if std::env::var("OPENAI_API_KEY").is_err() {
            // Mock client creation should work even without API key
            println!("Testing agent creation without API key...");
        }

        // Test basic agent creation (this doesn't require an API call)
        let openai_client = openai::Client::new("test-key");
        let _agent = openai_client
            .agent("gpt-3.5-turbo")
            .preamble("You are a helpful assistant.")
            .build();

        println!("✅ Agent creation test passed!");
        Ok(())
    }
}
