//! Standardized integration interfaces
//!
//! This module defines the core traits and structures for integrating
//! with external systems and services.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::Result;

/// Represents the capabilities of an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    /// Unique identifier for the service
    pub id: String,
    
    /// Human-readable name of the service
    pub name: String,
    
    /// Description of the service
    pub description: Option<String>,
    
    /// Version of the service
    pub version: Option<String>,
    
    /// Supported features as key-value pairs
    pub features: HashMap<String, String>,
    
    /// Authentication methods supported by the service
    pub auth_methods: Vec<String>,
    
    /// Rate limits information if available
    pub rate_limits: Option<RateLimitInfo>,
}

/// Rate limiting information for an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Maximum number of requests per time window
    pub requests_per_window: u32,
    
    /// Time window in seconds
    pub window_seconds: u32,
    
    /// Additional rate limit details as key-value pairs
    pub details: HashMap<String, String>,
}

/// Status of an external service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatus {
    /// Service is available and functioning normally
    Available,
    
    /// Service is available but experiencing degraded performance
    Degraded,
    
    /// Service is currently unavailable
    Unavailable,
    
    /// Service status is unknown
    Unknown,
}

/// Core trait for all external service integrations
#[async_trait]
pub trait ExternalService: Send + Sync {
    /// Get the unique identifier for this service
    fn id(&self) -> &str;
    
    /// Get the human-readable name of this service
    fn name(&self) -> &str;
    
    /// Get the capabilities of this service
    async fn capabilities(&self) -> Result<ServiceCapabilities>;
    
    /// Check the current status of the service
    async fn status(&self) -> Result<ServiceStatus>;
    
    /// Initialize the service connection
    async fn initialize(&self) -> Result<()>;
    
    /// Terminate the service connection
    async fn terminate(&self) -> Result<()>;
}

/// Trait for services that support data synchronization
#[async_trait]
pub trait DataSynchronization: ExternalService {
    /// Synchronize data with the external service
    async fn synchronize(&self) -> Result<SyncResult>;
    
    /// Get the last synchronization time
    async fn last_sync_time(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>>;
    
    /// Check if synchronization is currently in progress
    async fn is_syncing(&self) -> Result<bool>;
}

/// Result of a synchronization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Number of items added
    pub added: u32,
    
    /// Number of items updated
    pub updated: u32,
    
    /// Number of items deleted
    pub deleted: u32,
    
    /// Number of conflicts encountered
    pub conflicts: u32,
    
    /// Start time of the synchronization
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// End time of the synchronization
    pub end_time: chrono::DateTime<chrono::Utc>,
    
    /// Detailed messages about the synchronization
    pub messages: Vec<String>,
}

/// Trait for services that provide calendar functionality
#[async_trait]
pub trait CalendarService: ExternalService {
    /// List available calendars
    async fn list_calendars(&self) -> Result<Vec<Calendar>>;
    
    /// Get events for a specific calendar
    async fn get_events(&self, calendar_id: &str, start: &chrono::DateTime<chrono::Utc>, end: &chrono::DateTime<chrono::Utc>) -> Result<Vec<CalendarEvent>>;
    
    /// Create a new event
    async fn create_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<CalendarEvent>;
    
    /// Update an existing event
    async fn update_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<CalendarEvent>;
    
    /// Delete an event
    async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()>;
}

/// Represents a calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    /// Unique identifier for the calendar
    pub id: String,
    
    /// Human-readable name of the calendar
    pub name: String,
    
    /// Description of the calendar
    pub description: Option<String>,
    
    /// Color associated with the calendar
    pub color: Option<String>,
    
    /// Whether this calendar is the default
    pub is_default: bool,
    
    /// Access rights for this calendar
    pub access_rights: CalendarAccessRights,
}

/// Access rights for a calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarAccessRights {
    /// Whether the calendar can be read
    pub can_read: bool,
    
    /// Whether events can be added to the calendar
    pub can_add_events: bool,
    
    /// Whether events can be modified
    pub can_modify_events: bool,
    
    /// Whether events can be deleted
    pub can_delete_events: bool,
}

/// Represents a calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// Unique identifier for the event
    pub id: String,
    
    /// Title of the event
    pub title: String,
    
    /// Description of the event
    pub description: Option<String>,
    
    /// Location of the event
    pub location: Option<String>,
    
    /// Start time of the event
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// End time of the event
    pub end_time: chrono::DateTime<chrono::Utc>,
    
    /// Whether this is an all-day event
    pub all_day: bool,
    
    /// Recurrence rule for the event
    pub recurrence: Option<String>,
    
    /// Attendees of the event
    pub attendees: Vec<EventAttendee>,
    
    /// Reminders for the event
    pub reminders: Vec<EventReminder>,
}

/// Represents an attendee of a calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAttendee {
    /// Email address of the attendee
    pub email: String,
    
    /// Name of the attendee
    pub name: Option<String>,
    
    /// Response status of the attendee
    pub response_status: AttendeeResponseStatus,
}

/// Response status of an event attendee
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttendeeResponseStatus {
    /// Attendee has accepted the invitation
    Accepted,
    
    /// Attendee has tentatively accepted the invitation
    Tentative,
    
    /// Attendee has declined the invitation
    Declined,
    
    /// Attendee has not responded to the invitation
    NeedsAction,
}

/// Represents a reminder for a calendar event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReminder {
    /// Method of the reminder (e.g., "email", "notification")
    pub method: String,
    
    /// Minutes before the event when the reminder should trigger
    pub minutes_before: i32,
}

/// Trait for services that provide contact management functionality
#[async_trait]
pub trait ContactService: ExternalService {
    /// List available contact groups
    async fn list_contact_groups(&self) -> Result<Vec<ContactGroup>>;
    
    /// Get contacts for a specific group
    async fn get_contacts(&self, group_id: Option<&str>) -> Result<Vec<Contact>>;
    
    /// Create a new contact
    async fn create_contact(&self, contact: &Contact) -> Result<Contact>;
    
    /// Update an existing contact
    async fn update_contact(&self, contact: &Contact) -> Result<Contact>;
    
    /// Delete a contact
    async fn delete_contact(&self, contact_id: &str) -> Result<()>;
}

/// Represents a contact group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    /// Unique identifier for the group
    pub id: String,
    
    /// Name of the group
    pub name: String,
    
    /// Description of the group
    pub description: Option<String>,
    
    /// Number of contacts in the group
    pub contact_count: Option<u32>,
}

/// Represents a contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Unique identifier for the contact
    pub id: String,
    
    /// First name of the contact
    pub first_name: Option<String>,
    
    /// Last name of the contact
    pub last_name: Option<String>,
    
    /// Display name of the contact
    pub display_name: String,
    
    /// Email addresses of the contact
    pub emails: Vec<ContactEmail>,
    
    /// Phone numbers of the contact
    pub phones: Vec<ContactPhone>,
    
    /// Addresses of the contact
    pub addresses: Vec<ContactAddress>,
    
    /// Organizations associated with the contact
    pub organizations: Vec<ContactOrganization>,
    
    /// Notes about the contact
    pub notes: Option<String>,
    
    /// Photo/avatar of the contact
    pub photo_url: Option<String>,
    
    /// Groups this contact belongs to
    pub group_ids: Vec<String>,
}

/// Represents an email address of a contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEmail {
    /// The email address
    pub address: String,
    
    /// Type of the email address (e.g., "home", "work")
    pub type_: String,
    
    /// Whether this is the primary email address
    pub is_primary: bool,
}

/// Represents a phone number of a contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactPhone {
    /// The phone number
    pub number: String,
    
    /// Type of the phone number (e.g., "mobile", "home", "work")
    pub type_: String,
    
    /// Whether this is the primary phone number
    pub is_primary: bool,
}

/// Represents a physical address of a contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactAddress {
    /// Street address
    pub street: Option<String>,
    
    /// City
    pub city: Option<String>,
    
    /// State or province
    pub state: Option<String>,
    
    /// Postal code
    pub postal_code: Option<String>,
    
    /// Country
    pub country: Option<String>,
    
    /// Type of the address (e.g., "home", "work")
    pub type_: String,
    
    /// Whether this is the primary address
    pub is_primary: bool,
}

/// Represents an organization associated with a contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactOrganization {
    /// Name of the organization
    pub name: String,
    
    /// Title of the contact within the organization
    pub title: Option<String>,
    
    /// Department of the contact within the organization
    pub department: Option<String>,
    
    /// Whether this is the primary organization
    pub is_primary: bool,
}

/// Factory for creating external service instances
pub trait ServiceFactory: Send + Sync {
    /// Create a new instance of an external service
    fn create_service(&self, service_type: &str, config: &HashMap<String, String>) -> Result<Box<dyn ExternalService>>;
    
    /// Get the list of supported service types
    fn supported_service_types(&self) -> Vec<String>;
}