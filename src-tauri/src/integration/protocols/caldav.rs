//! CalDAV protocol implementation
//!
//! This module provides an implementation of the CalDAV protocol
//! for calendar synchronization with external services.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use quick_xml::Writer;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use crate::error::{Error, ErrorKind, Result};
use crate::integration::interfaces::{CalendarEvent, Calendar, CalendarAccessRights};
use crate::integration::auth::{AuthToken, AuthMethod};
use crate::integration::protocols::common::{
    WebDavClient, WebDavConfig, WebDavProtocolClient, 
    Resource, ResourceType, Depth, PropertyName, Property, PropertyValue
};

/// iCalendar component types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarComponentType {
    /// Calendar event (VEVENT)
    Event,
    
    /// Calendar todo (VTODO)
    Todo,
    
    /// Calendar journal entry (VJOURNAL)
    Journal,
    
    /// Free/busy time (VFREEBUSY)
    FreeBusy,
    
    /// Timezone information (VTIMEZONE)
    Timezone,
    
    /// Alarm (VALARM)
    Alarm,
}

impl ICalendarComponentType {
    /// Get the iCalendar component type name
    pub fn name(&self) -> &'static str {
        match self {
            ICalendarComponentType::Event => "VEVENT",
            ICalendarComponentType::Todo => "VTODO",
            ICalendarComponentType::Journal => "VJOURNAL",
            ICalendarComponentType::FreeBusy => "VFREEBUSY",
            ICalendarComponentType::Timezone => "VTIMEZONE",
            ICalendarComponentType::Alarm => "VALARM",
        }
    }
}

/// CalDAV client configuration
#[derive(Debug, Clone)]
pub struct CalDavConfig {
    /// WebDAV configuration
    pub webdav_config: WebDavConfig,
    
    /// Default calendar URL (if known)
    pub default_calendar_url: Option<String>,
    
    /// User principal URL (if known)
    pub principal_url: Option<String>,
}

impl Default for CalDavConfig {
    fn default() -> Self {
        Self {
            webdav_config: WebDavConfig::default(),
            default_calendar_url: None,
            principal_url: None,
        }
    }
}

/// CalDAV client for interacting with CalDAV servers
pub struct CalDavClient {
    /// WebDAV client
    webdav: WebDavClient,
    
    /// CalDAV configuration
    config: CalDavConfig,
}

impl CalDavClient {
    /// Create a new CalDAV client
    pub fn new(config: CalDavConfig) -> Result<Self> {
        let webdav = WebDavClient::new(config.webdav_config.clone())?;
        
        Ok(Self {
            webdav,
            config,
        })
    }
    
    /// Find the principal URL for the current user
    pub async fn find_principal_url(&self) -> Result<String> {
        // If we already have a principal URL, return it
        if let Some(url) = &self.config.principal_url {
            return Ok(url.clone());
        }
        
        // Otherwise, try to discover it
        let props = vec![
            PropertyName::dav("current-user-principal"),
            PropertyName::dav("principal-URL"),
        ];
        
        let resources = self.webdav.propfind("/", Depth::Zero, &props).await?;
        
        if let Some(resource) = resources.first() {
            // Try to get the current-user-principal property
            if let Some(PropertyValue::Text(url)) = resource.properties.get(&PropertyName::dav("current-user-principal")) {
                return Ok(url.clone());
            }
            
            // Try to get the principal-URL property
            if let Some(PropertyValue::Text(url)) = resource.properties.get(&PropertyName::dav("principal-URL")) {
                return Ok(url.clone());
            }
        }
        
        Err(Error::new(
            ErrorKind::NotFound,
            "Could not find principal URL"
        ))
    }
    
    /// Find calendar home set URL
    pub async fn find_calendar_home_set(&self) -> Result<String> {
        // First, find the principal URL
        let principal_url = self.find_principal_url().await?;
        
        // Then, find the calendar home set
        let props = vec![
            PropertyName::caldav("calendar-home-set"),
        ];
        
        let resources = self.webdav.propfind(&principal_url, Depth::Zero, &props).await?;
        
        if let Some(resource) = resources.first() {
            if let Some(PropertyValue::Text(url)) = resource.properties.get(&PropertyName::caldav("calendar-home-set")) {
                return Ok(url.clone());
            }
        }
        
        Err(Error::new(
            ErrorKind::NotFound,
            "Could not find calendar home set"
        ))
    }
    
    /// Convert a WebDAV resource to a Calendar
    fn resource_to_calendar(&self, resource: &Resource) -> Result<Calendar> {
        // Get the calendar display name
        let display_name = if let Some(PropertyValue::Text(name)) = resource.properties.get(&PropertyName::dav("displayname")) {
            name.clone()
        } else {
            // Use the last part of the URL as the name
            let parts: Vec<&str> = resource.url.split('/').collect();
            parts.last().unwrap_or(&"Calendar").to_string()
        };
        
        // Get the calendar color
        let color = if let Some(PropertyValue::Text(color)) = resource.properties.get(&PropertyName::new("http://apple.com/ns/ical/", "calendar-color")) {
            Some(color.clone())
        } else {
            None
        };
        
        // Get the calendar description
        let description = if let Some(PropertyValue::Text(desc)) = resource.properties.get(&PropertyName::caldav("calendar-description")) {
            Some(desc.clone())
        } else {
            None
        };
        
        // Determine if this is the default calendar
        let is_default = if let Some(PropertyValue::Text(default)) = resource.properties.get(&PropertyName::new("http://apple.com/ns/ical/", "calendar-default")) {
            default == "1" || default.to_lowercase() == "true"
        } else {
            false
        };
        
        // Get the calendar access rights
        let access_rights = CalendarAccessRights {
            can_read: true, // If we can see it, we can read it
            can_add_events: true, // Assume we can add events
            can_modify_events: true, // Assume we can modify events
            can_delete_events: true, // Assume we can delete events
        };
        
        Ok(Calendar {
            id: resource.url.clone(),
            name: display_name,
            description,
            color,
            is_default,
            access_rights,
        })
    }
    
    /// Build a calendar query request body
    fn build_calendar_query(&self, start: &DateTime<Utc>, end: &DateTime<Utc>, component_type: ICalendarComponentType) -> Result<String> {
        let mut writer = Writer::new(Vec::new());
        
        // Write the XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write XML declaration: {}", e)
                )
            })?;
        
        // Start the calendar-query element
        let mut calendar_query_elem = BytesStart::new("calendar-query");
        calendar_query_elem.push_attribute(("xmlns", "urn:ietf:params:xml:ns:caldav"));
        calendar_query_elem.push_attribute(("xmlns:d", "DAV:"));
        writer.write_event(Event::Start(calendar_query_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write calendar-query element: {}", e)
                )
            })?;
        
        // Start the prop element
        let prop_elem = BytesStart::new("d:prop");
        writer.write_event(Event::Start(prop_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write prop element: {}", e)
                )
            })?;
        
        // Add the getetag property
        let getetag_elem = BytesStart::new("d:getetag");
        writer.write_event(Event::Empty(getetag_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write getetag element: {}", e)
                )
            })?;
        
        // Add the calendar-data property
        let calendar_data_elem = BytesStart::new("calendar-data");
        writer.write_event(Event::Empty(calendar_data_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write calendar-data element: {}", e)
                )
            })?;
        
        // End the prop element
        writer.write_event(Event::End(BytesEnd::new("d:prop")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end prop element: {}", e)
                )
            })?;
        
        // Start the filter element
        let filter_elem = BytesStart::new("filter");
        writer.write_event(Event::Start(filter_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write filter element: {}", e)
                )
            })?;
        
        // Start the comp-filter element for VCALENDAR
        let mut comp_filter_elem = BytesStart::new("comp-filter");
        comp_filter_elem.push_attribute(("name", "VCALENDAR"));
        writer.write_event(Event::Start(comp_filter_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write comp-filter element: {}", e)
                )
            })?;
        
        // Start the comp-filter element for the component type
        let mut comp_filter_elem = BytesStart::new("comp-filter");
        comp_filter_elem.push_attribute(("name", component_type.name()));
        writer.write_event(Event::Start(comp_filter_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write comp-filter element: {}", e)
                )
            })?;
        
        // Start the time-range element
        let mut time_range_elem = BytesStart::new("time-range");
        time_range_elem.push_attribute(("start", &format!("{}", start.format("%Y%m%dT%H%M%SZ"))));
        time_range_elem.push_attribute(("end", &format!("{}", end.format("%Y%m%dT%H%M%SZ"))));
        writer.write_event(Event::Empty(time_range_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write time-range element: {}", e)
                )
            })?;
        
        // End the component type comp-filter element
        writer.write_event(Event::End(BytesEnd::new("comp-filter")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end comp-filter element: {}", e)
                )
            })?;
        
        // End the VCALENDAR comp-filter element
        writer.write_event(Event::End(BytesEnd::new("comp-filter")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end comp-filter element: {}", e)
                )
            })?;
        
        // End the filter element
        writer.write_event(Event::End(BytesEnd::new("filter")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end filter element: {}", e)
                )
            })?;
        
        // End the calendar-query element
        writer.write_event(Event::End(BytesEnd::new("calendar-query")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end calendar-query element: {}", e)
                )
            })?;
        
        // Convert to string
        let xml = String::from_utf8(writer.into_inner())
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to convert XML to string: {}", e)
                )
            })?;
        
        Ok(xml)
    }
    
    /// Parse iCalendar data to extract events
    fn parse_icalendar(&self, ical_data: &str) -> Result<Vec<CalendarEvent>> {
        // This is a simplified implementation that would need to be expanded
        // with a proper iCalendar parser in a real implementation
        
        let mut events = Vec::new();
        let mut current_event: Option<CalendarEvent> = None;
        let mut in_event = false;
        
        for line in ical_data.lines() {
            let line = line.trim();
            
            if line == "BEGIN:VEVENT" {
                in_event = true;
                current_event = Some(CalendarEvent {
                    id: String::new(),
                    title: String::new(),
                    description: None,
                    location: None,
                    start_time: Utc::now(),
                    end_time: Utc::now(),
                    all_day: false,
                    recurrence: None,
                    attendees: Vec::new(),
                    reminders: Vec::new(),
                });
            } else if line == "END:VEVENT" {
                in_event = false;
                if let Some(event) = current_event.take() {
                    events.push(event);
                }
            } else if in_event {
                if let Some(event) = &mut current_event {
                    if line.starts_with("UID:") {
                        event.id = line[4..].to_string();
                    } else if line.starts_with("SUMMARY:") {
                        event.title = line[8..].to_string();
                    } else if line.starts_with("DESCRIPTION:") {
                        event.description = Some(line[12..].to_string());
                    } else if line.starts_with("LOCATION:") {
                        event.location = Some(line[9..].to_string());
                    } else if line.starts_with("DTSTART") {
                        // Parse start time (simplified)
                        if line.contains("VALUE=DATE") {
                            event.all_day = true;
                        }
                        // In a real implementation, parse the datetime properly
                    } else if line.starts_with("DTEND") {
                        // Parse end time (simplified)
                        // In a real implementation, parse the datetime properly
                    } else if line.starts_with("RRULE:") {
                        event.recurrence = Some(line[6..].to_string());
                    }
                    // In a real implementation, parse attendees, reminders, etc.
                }
            }
        }
        
        Ok(events)
    }
    
    /// Create iCalendar data from an event
    fn create_icalendar(&self, event: &CalendarEvent) -> Result<String> {
        let mut ical = String::new();
        
        // Add iCalendar header
        ical.push_str("BEGIN:VCALENDAR\r\n");
        ical.push_str("VERSION:2.0\r\n");
        ical.push_str("PRODID:-//evo-pro//CalDAV Client//EN\r\n");
        
        // Add event
        ical.push_str("BEGIN:VEVENT\r\n");
        
        // Add UID
        ical.push_str(&format!("UID:{}\r\n", event.id));
        
        // Add created and last modified timestamps
        let now = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        ical.push_str(&format!("DTSTAMP:{}\r\n", now));
        ical.push_str(&format!("CREATED:{}\r\n", now));
        ical.push_str(&format!("LAST-MODIFIED:{}\r\n", now));
        
        // Add summary (title)
        ical.push_str(&format!("SUMMARY:{}\r\n", event.title));
        
        // Add description if present
        if let Some(description) = &event.description {
            ical.push_str(&format!("DESCRIPTION:{}\r\n", description));
        }
        
        // Add location if present
        if let Some(location) = &event.location {
            ical.push_str(&format!("LOCATION:{}\r\n", location));
        }
        
        // Add start time
        if event.all_day {
            ical.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n", 
                event.start_time.format("%Y%m%d").to_string()));
        } else {
            ical.push_str(&format!("DTSTART:{}\r\n", 
                event.start_time.format("%Y%m%dT%H%M%SZ").to_string()));
        }
        
        // Add end time
        if event.all_day {
            ical.push_str(&format!("DTEND;VALUE=DATE:{}\r\n", 
                event.end_time.format("%Y%m%d").to_string()));
        } else {
            ical.push_str(&format!("DTEND:{}\r\n", 
                event.end_time.format("%Y%m%dT%H%M%SZ").to_string()));
        }
        
        // Add recurrence rule if present
        if let Some(recurrence) = &event.recurrence {
            ical.push_str(&format!("RRULE:{}\r\n", recurrence));
        }
        
        // Add attendees
        for attendee in &event.attendees {
            let status = match attendee.response_status {
                crate::integration::interfaces::AttendeeResponseStatus::Accepted => "ACCEPTED",
                crate::integration::interfaces::AttendeeResponseStatus::Tentative => "TENTATIVE",
                crate::integration::interfaces::AttendeeResponseStatus::Declined => "DECLINED",
                crate::integration::interfaces::AttendeeResponseStatus::NeedsAction => "NEEDS-ACTION",
            };
            
            let mut attendee_str = format!("ATTENDEE;PARTSTAT={}:mailto:{}", status, attendee.email);
            if let Some(name) = &attendee.name {
                attendee_str = format!("ATTENDEE;CN={};PARTSTAT={}:mailto:{}", name, status, attendee.email);
            }
            
            ical.push_str(&format!("{}\r\n", attendee_str));
        }
        
        // Add reminders (alarms)
        for reminder in &event.reminders {
            ical.push_str("BEGIN:VALARM\r\n");
            ical.push_str(&format!("ACTION:{}\r\n", reminder.method.to_uppercase()));
            ical.push_str(&format!("TRIGGER:-PT{}M\r\n", reminder.minutes_before));
            ical.push_str("END:VALARM\r\n");
        }
        
        // End event
        ical.push_str("END:VEVENT\r\n");
        
        // End calendar
        ical.push_str("END:VCALENDAR\r\n");
        
        Ok(ical)
    }
}

#[async_trait]
impl WebDavProtocolClient for CalDavClient {
    fn webdav_client(&self) -> &WebDavClient {
        &self.webdav
    }
    
    async fn discover_resources(&self, resource_type: ResourceType) -> Result<Vec<Resource>> {
        // For CalDAV, we're interested in calendar collections
        if resource_type != ResourceType::Calendar {
            return Ok(Vec::new());
        }
        
        // Find the calendar home set
        let calendar_home = self.find_calendar_home_set().await?;
        
        // Get all collections in the calendar home
        let props = vec![
            PropertyName::dav("resourcetype"),
            PropertyName::dav("displayname"),
            PropertyName::caldav("calendar-description"),
            PropertyName::new("http://apple.com/ns/ical/", "calendar-color"),
            PropertyName::new("http://apple.com/ns/ical/", "calendar-default"),
        ];
        
        let resources = self.webdav.propfind(&calendar_home, Depth::One, &props).await?;
        
        // Filter for calendar collections
        let calendars = resources.into_iter()
            .filter(|r| r.resource_type == ResourceType::Calendar)
            .collect();
        
        Ok(calendars)
    }
    
    async fn get_resource(&self, url: &str) -> Result<Resource> {
        let props = vec![
            PropertyName::dav("resourcetype"),
            PropertyName::dav("displayname"),
            PropertyName::caldav("calendar-description"),
            PropertyName::new("http://apple.com/ns/ical/", "calendar-color"),
            PropertyName::new("http://apple.com/ns/ical/", "calendar-default"),
        ];
        
        let resources = self.webdav.propfind(url, Depth::Zero, &props).await?;
        
        if let Some(resource) = resources.first() {
            Ok(resource.clone())
        } else {
            Err(Error::new(
                ErrorKind::NotFound,
                &format!("Resource not found: {}", url)
            ))
        }
    }
    
    async fn create_collection(&self, url: &str) -> Result<()> {
        // Create the collection
        self.webdav.mkcol(url).await?;
        
        // Set the resourcetype to calendar
        let props = vec![
            Property {
                name: PropertyName::dav("resourcetype"),
                value: PropertyValue::Element(
                    "<D:collection xmlns:D=\"DAV:\"/><C:calendar xmlns:C=\"urn:ietf:params:xml:ns:caldav\"/>".as_bytes().to_vec()
                ),
            },
        ];
        
        self.webdav.proppatch(url, &props).await?;
        
        Ok(())
    }
    
    async fn delete_resource(&self, url: &str) -> Result<()> {
        self.webdav.delete(url).await
    }
}

#[async_trait]
impl crate::integration::interfaces::CalendarService for CalDavClient {
    async fn list_calendars(&self) -> Result<Vec<Calendar>> {
        let resources = self.discover_resources(ResourceType::Calendar).await?;
        
        let mut calendars = Vec::new();
        for resource in resources {
            let calendar = self.resource_to_calendar(&resource)?;
            calendars.push(calendar);
        }
        
        Ok(calendars)
    }
    
    async fn get_events(&self, calendar_id: &str, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Result<Vec<CalendarEvent>> {
        // Build the calendar query
        let query = self.build_calendar_query(start, end, ICalendarComponentType::Event)?;
        
        // Send the REPORT request
        let response = self.webdav.report(calendar_id, query).await?;
        
        // Parse the response to extract iCalendar data
        // This is a simplified implementation - in a real implementation,
        // you would need to parse the XML response to extract the calendar-data elements
        
        // For now, we'll just assume the response contains iCalendar data
        let events = self.parse_icalendar(&response)?;
        
        Ok(events)
    }
    
    async fn create_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<CalendarEvent> {
        // Create iCalendar data
        let ical_data = self.create_icalendar(event)?;
        
        // Create a URL for the event
        let event_url = format!("{}/{}.ics", calendar_id, event.id);
        
        // Put the event
        self.webdav.put(&event_url, "text/calendar; charset=utf-8", ical_data.into_bytes()).await?;
        
        // Return the event (in a real implementation, you might want to fetch the event to get server-assigned properties)
        Ok(event.clone())
    }
    
    async fn update_event(&self, calendar_id: &str, event: &CalendarEvent) -> Result<CalendarEvent> {
        // Create iCalendar data
        let ical_data = self.create_icalendar(event)?;
        
        // Create a URL for the event
        let event_url = format!("{}/{}.ics", calendar_id, event.id);
        
        // Put the event
        self.webdav.put(&event_url, "text/calendar; charset=utf-8", ical_data.into_bytes()).await?;
        
        // Return the event (in a real implementation, you might want to fetch the event to get server-assigned properties)
        Ok(event.clone())
    }
    
    async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        // Create a URL for the event
        let event_url = format!("{}/{}.ics", calendar_id, event_id);
        
        // Delete the event
        self.webdav.delete(&event_url).await
    }
}