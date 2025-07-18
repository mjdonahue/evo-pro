// Service layer modules
pub mod agent;
pub mod composition;
pub mod consent_management;
pub mod conversation;
pub mod core;
pub mod data_deletion_verification;
pub mod data_export;
pub mod data_minimization;
pub mod data_retention;
pub mod data_usage_reporting;
pub mod events;
pub mod logging;
pub mod message;
pub mod middleware;
pub mod plan;
pub mod plugin_marketplace;
pub mod privacy_analytics;
pub mod privacy_policy;
pub mod security;
pub mod task;
pub mod traits;
pub mod transaction;

// Re-exports for convenience
pub use agent::AgentService;
pub use composition::*;
pub use conversation::ConversationService;
pub use core::*;
pub use data_deletion_verification::{DataDeletionVerificationService, verify_data_deletion, generate_deletion_certificate};
pub use data_export::{DataExportService, export_user_data};
pub use data_minimization::DataMinimizationService;
pub use data_retention::{DataRetentionService, get_retention_policy, set_retention_policy, apply_retention_policy};
pub use consent_management::{ConsentManagementService, get_user_consent, update_user_consent};
pub use data_usage_reporting::{DataUsageReportingService, generate_data_usage_report, update_data_preferences};
pub use events::EventService;
pub use logging::*;
pub use message::MessageService;
pub use middleware::*;
pub use plan::PlanService;
pub use plugin_marketplace::{
    get_plugin_marketplace_sources,
    get_plugin_marketplace_entries,
    search_plugin_marketplace,
    install_plugin_from_marketplace,
    uninstall_plugin_from_marketplace,
    update_plugin_from_marketplace,
    refresh_plugin_marketplace
};
pub use privacy_analytics::PrivacyAnalyticsService;
pub use privacy_policy::PrivacyPolicyService;
pub use security::SecurityService;
pub use task::TaskService;
pub use traits::*;
pub use transaction::*;
