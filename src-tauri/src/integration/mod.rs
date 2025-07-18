//! External system integration module
//! 
//! This module provides standardized interfaces and implementations for
//! integrating with external systems and services.

mod interfaces;
mod auth;
mod rate_limiting;
mod monitoring;
pub mod protocols;

pub use interfaces::*;
pub use auth::*;
pub use rate_limiting::*;
pub use monitoring::*;

use std::sync::{Arc, Mutex, Once};

/// Global monitoring service instance
static MONITORING_SERVICE_INIT: Once = Once::new();
static mut MONITORING_SERVICE: Option<Arc<Mutex<MonitoringService>>> = None;

/// Get the global monitoring service instance
pub fn get_monitoring_service() -> Arc<Mutex<MonitoringService>> {
    unsafe {
        MONITORING_SERVICE_INIT.call_once(|| {
            // Create a new HTTP health check strategy
            let health_checker = Arc::new(HttpHealthCheck::new());

            // Create a new monitoring service
            let mut monitoring_service = MonitoringService::new(health_checker);

            // Add default alert handlers
            setup_default_alert_handlers(&mut monitoring_service);

            MONITORING_SERVICE = Some(Arc::new(Mutex::new(monitoring_service)));
        });

        MONITORING_SERVICE.clone().unwrap()
    }
}

/// Set up default alert handlers for the monitoring service
fn setup_default_alert_handlers(monitoring_service: &mut MonitoringService) {
    // Add console alert handler
    monitoring_service.add_alert_handler(Arc::new(ConsoleAlertHandler));

    // Additional alert handlers can be added here
    // For example, email, SMS, or integration with external monitoring systems
}

/// Initialize the integration module
pub fn init() {
    tracing::info!("Initializing integration module");

    // Initialize the monitoring service
    let _monitoring_service = get_monitoring_service();
    tracing::info!("Integration monitoring service initialized");
}
