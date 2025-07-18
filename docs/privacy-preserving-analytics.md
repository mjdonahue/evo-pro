# Privacy-Preserving Analytics

This document describes the privacy-preserving analytics system implemented in the evo-pro project.

## Overview

The privacy-preserving analytics system is designed to collect usage data while respecting user privacy through:

1. **Explicit Consent**: Analytics collection requires explicit user consent, which is granular and specific to different types of data.
2. **Data Minimization**: Only necessary data is collected, and multiple levels of anonymization are applied.
3. **Retention Policies**: Data is automatically deleted after a configurable retention period.
4. **Differential Privacy**: Advanced anonymization techniques are used to protect user privacy.

## Architecture

The system consists of the following components:

1. **PrivacyAnalyticsService**: The core service that manages analytics collection, anonymization, and reporting.
2. **Analytics Events**: Structured data representing user actions and system events.
3. **User Consent Management**: A system for tracking and enforcing user consent preferences.
4. **Anonymization Pipeline**: Multiple levels of anonymization techniques applied to collected data.
5. **Reporting System**: Aggregated analytics reports that preserve privacy.

## Anonymization Levels

The system supports four levels of anonymization:

1. **None**: No anonymization is applied. This is only used for data where the user has explicitly consented.
2. **Basic**: Direct identifiers (names, emails, IDs) are removed from the data.
3. **Advanced**: Implements k-anonymity by generalizing data (rounding numbers, reducing timestamp precision).
4. **Full**: Implements differential privacy by adding noise to data and only keeping aggregate information.

## User Consent

User consent is granular and specific to different types of analytics:

1. **Feature Usage**: Tracking which features are used and how often.
2. **Performance**: Measuring application performance metrics.
3. **Error Reporting**: Collecting information about errors and crashes.
4. **User Interface**: Tracking how users interact with the UI.

Users can update their consent preferences at any time, and the system will immediately respect these changes.

## Data Collection

The system collects the following types of events:

1. **Feature Usage**: When users interact with specific features.
2. **Performance**: Application performance metrics.
3. **Error**: Information about errors and exceptions.
4. **User Interface**: User interactions with the UI.
5. **Session**: Session start/end and duration.
6. **Custom**: Application-specific events.

For each event, the following information is recorded:

- Event type and name
- Anonymized session ID
- Timestamp
- Anonymized properties specific to the event
- Consent status
- Anonymization level applied

## Privacy Protections

The system implements several privacy protections:

1. **Opt-In by Default**: Analytics collection is disabled by default and requires explicit user consent.
2. **Data Minimization**: Only necessary data is collected, and sensitive fields are removed.
3. **Anonymization**: Multiple levels of anonymization are applied based on consent.
4. **Retention Policies**: Data is automatically deleted after a configurable period (default: 90 days).
5. **Development Mode Protection**: Analytics collection is disabled in development mode by default.

## Usage

### Initializing the Service

```rust
let db = DatabaseManager::get_instance().await?;
let analytics_service = PrivacyAnalyticsService::new(db.clone());
analytics_service.initialize().await?;
```

### Tracking Events

```rust
analytics_service.track_event(
    AnalyticsEventType::FeatureUsage,
    "feature_name_used",
    session_id,
    json!({
        "feature": "feature_name",
        "action": "click",
        "duration_ms": 123
    }),
    Some(&user_id)
).await?;
```

### Managing User Consent

```rust
// Update user consent
analytics_service.update_consent(
    &user_id,
    true,  // feature usage
    true,  // performance
    true,  // error reporting
    false  // user interface
).await?;

// Get current consent
let consent = analytics_service.get_consent(&user_id).await?;
```

### Generating Reports

```rust
// Generate a report for the last 30 days
let start_date = Utc::now() - Duration::days(30);
let report = analytics_service.generate_report(
    Some(AnalyticsEventType::FeatureUsage),
    Some(start_date),
    None
).await?;
```

## Integration with Frontend

The analytics service can be integrated with the frontend through Tauri commands:

```typescript
// Track an event from the frontend
await invoke('track_analytics_event', {
  eventType: 'feature_usage',
  eventName: 'feature_name_used',
  properties: {
    feature: 'feature_name',
    action: 'click',
    duration_ms: 123
  }
});

// Update user consent from the frontend
await invoke('update_analytics_consent', {
  featureUsage: true,
  performance: true,
  errorReporting: true,
  userInterface: false
});
```

## Best Practices

When using the analytics system, follow these best practices:

1. **Be Transparent**: Clearly inform users about what data is collected and why.
2. **Respect User Choice**: Always honor user consent preferences.
3. **Collect Minimal Data**: Only collect data that is necessary for the intended purpose.
4. **Use Appropriate Anonymization**: Choose the appropriate anonymization level based on the sensitivity of the data.
5. **Regular Audits**: Regularly audit the analytics system to ensure it's functioning as expected and respecting privacy.

## Future Improvements

Potential future improvements to the analytics system include:

1. Integration with a proper differential privacy library for more robust privacy guarantees
2. More sophisticated anonymization techniques for specific data types
3. Enhanced visualization and reporting capabilities
4. Integration with privacy impact assessments
5. Automated privacy compliance checks