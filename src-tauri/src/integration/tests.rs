//! Tests for the integration module

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;

    #[test]
    fn test_monitoring_service_initialization() {
        // Initialize the integration module
        init();

        // Get the monitoring service
        let monitoring_service = get_monitoring_service();

        // Check that the monitoring service is properly initialized
        assert!(monitoring_service.lock().unwrap().alert_handlers.len() > 0);
    }

    #[tokio::test]
    async fn test_monitoring_service_alerts() {
        // Initialize the integration module
        init();

        // Get the monitoring service
        let monitoring_service = get_monitoring_service();
        let service = monitoring_service.lock().unwrap();

        // Create a test alert
        let alert = service.create_alert(
            "test-service",
            AlertSeverity::Warning,
            "Test Alert",
            "This is a test alert",
        ).await.unwrap();

        // Check that the alert was created
        assert_eq!(alert.service_id, "test-service");
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.title, "Test Alert");
        assert_eq!(alert.description, "This is a test alert");
        assert!(!alert.is_resolved());

        // Get active alerts
        let active_alerts = service.get_active_alerts().unwrap();
        assert!(active_alerts.len() > 0);

        // Resolve the alert
        let resolved_alert = service.resolve_alert(&alert.id).await.unwrap();
        assert!(resolved_alert.is_resolved());
    }
}