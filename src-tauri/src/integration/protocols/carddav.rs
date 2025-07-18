//! CardDAV protocol implementation
//!
//! This module provides an implementation of the CardDAV protocol
//! for contact synchronization with external services.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use quick_xml::Writer;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use crate::error::{Error, ErrorKind, Result};
use crate::integration::interfaces::{Contact, ContactGroup, ContactEmail, ContactPhone, ContactAddress, ContactOrganization};
use crate::integration::auth::{AuthToken, AuthMethod};
use crate::integration::protocols::common::{
    WebDavClient, WebDavConfig, WebDavProtocolClient, 
    Resource, ResourceType, Depth, PropertyName, Property, PropertyValue
};

/// vCard version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VCardVersion {
    /// vCard 3.0
    V3,
    
    /// vCard 4.0
    V4,
}

impl VCardVersion {
    /// Get the vCard version string
    pub fn version_string(&self) -> &'static str {
        match self {
            VCardVersion::V3 => "3.0",
            VCardVersion::V4 => "4.0",
        }
    }
    
    /// Get the content type for this vCard version
    pub fn content_type(&self) -> &'static str {
        match self {
            VCardVersion::V3 => "text/vcard",
            VCardVersion::V4 => "text/vcard; version=4.0",
        }
    }
}

/// CardDAV client configuration
#[derive(Debug, Clone)]
pub struct CardDavConfig {
    /// WebDAV configuration
    pub webdav_config: WebDavConfig,
    
    /// Default address book URL (if known)
    pub default_addressbook_url: Option<String>,
    
    /// User principal URL (if known)
    pub principal_url: Option<String>,
    
    /// vCard version to use
    pub vcard_version: VCardVersion,
}

impl Default for CardDavConfig {
    fn default() -> Self {
        Self {
            webdav_config: WebDavConfig::default(),
            default_addressbook_url: None,
            principal_url: None,
            vcard_version: VCardVersion::V3,
        }
    }
}

/// CardDAV client for interacting with CardDAV servers
pub struct CardDavClient {
    /// WebDAV client
    webdav: WebDavClient,
    
    /// CardDAV configuration
    config: CardDavConfig,
}

impl CardDavClient {
    /// Create a new CardDAV client
    pub fn new(config: CardDavConfig) -> Result<Self> {
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
    
    /// Find address book home set URL
    pub async fn find_addressbook_home_set(&self) -> Result<String> {
        // First, find the principal URL
        let principal_url = self.find_principal_url().await?;
        
        // Then, find the address book home set
        let props = vec![
            PropertyName::carddav("addressbook-home-set"),
        ];
        
        let resources = self.webdav.propfind(&principal_url, Depth::Zero, &props).await?;
        
        if let Some(resource) = resources.first() {
            if let Some(PropertyValue::Text(url)) = resource.properties.get(&PropertyName::carddav("addressbook-home-set")) {
                return Ok(url.clone());
            }
        }
        
        Err(Error::new(
            ErrorKind::NotFound,
            "Could not find address book home set"
        ))
    }
    
    /// Convert a WebDAV resource to a ContactGroup (address book)
    fn resource_to_contact_group(&self, resource: &Resource) -> Result<ContactGroup> {
        // Get the address book display name
        let display_name = if let Some(PropertyValue::Text(name)) = resource.properties.get(&PropertyName::dav("displayname")) {
            name.clone()
        } else {
            // Use the last part of the URL as the name
            let parts: Vec<&str> = resource.url.split('/').collect();
            parts.last().unwrap_or(&"Address Book").to_string()
        };
        
        // Get the address book description
        let description = if let Some(PropertyValue::Text(desc)) = resource.properties.get(&PropertyName::carddav("addressbook-description")) {
            Some(desc.clone())
        } else {
            None
        };
        
        // Get the contact count if available
        let contact_count = if let Some(PropertyValue::Text(count)) = resource.properties.get(&PropertyName::new("http://apple.com/ns/carddav/", "contacts-count")) {
            count.parse::<u32>().ok()
        } else {
            None
        };
        
        Ok(ContactGroup {
            id: resource.url.clone(),
            name: display_name,
            description,
            contact_count,
        })
    }
    
    /// Build an address book query request body
    fn build_addressbook_query(&self) -> Result<String> {
        let mut writer = Writer::new(Vec::new());
        
        // Write the XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write XML declaration: {}", e)
                )
            })?;
        
        // Start the addressbook-query element
        let mut addressbook_query_elem = BytesStart::new("addressbook-query");
        addressbook_query_elem.push_attribute(("xmlns", "urn:ietf:params:xml:ns:carddav"));
        addressbook_query_elem.push_attribute(("xmlns:d", "DAV:"));
        writer.write_event(Event::Start(addressbook_query_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write addressbook-query element: {}", e)
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
        
        // Add the address-data property
        let address_data_elem = BytesStart::new("address-data");
        writer.write_event(Event::Empty(address_data_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write address-data element: {}", e)
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
        
        // End the addressbook-query element
        writer.write_event(Event::End(BytesEnd::new("addressbook-query")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end addressbook-query element: {}", e)
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
    
    /// Parse vCard data to extract contacts
    fn parse_vcard(&self, vcard_data: &str) -> Result<Vec<Contact>> {
        // This is a simplified implementation that would need to be expanded
        // with a proper vCard parser in a real implementation
        
        let mut contacts = Vec::new();
        let mut current_contact: Option<Contact> = None;
        let mut in_vcard = false;
        
        for line in vcard_data.lines() {
            let line = line.trim();
            
            if line == "BEGIN:VCARD" {
                in_vcard = true;
                current_contact = Some(Contact {
                    id: String::new(),
                    first_name: None,
                    last_name: None,
                    display_name: String::new(),
                    emails: Vec::new(),
                    phones: Vec::new(),
                    addresses: Vec::new(),
                    organizations: Vec::new(),
                    notes: None,
                    photo_url: None,
                    group_ids: Vec::new(),
                });
            } else if line == "END:VCARD" {
                in_vcard = false;
                if let Some(contact) = current_contact.take() {
                    contacts.push(contact);
                }
            } else if in_vcard {
                if let Some(contact) = &mut current_contact {
                    if line.starts_with("UID:") {
                        contact.id = line[4..].to_string();
                    } else if line.starts_with("FN:") {
                        contact.display_name = line[3..].to_string();
                    } else if line.starts_with("N:") {
                        // Parse name components (simplified)
                        let parts: Vec<&str> = line[2..].split(';').collect();
                        if parts.len() >= 2 {
                            contact.last_name = Some(parts[0].to_string());
                            contact.first_name = Some(parts[1].to_string());
                        }
                    } else if line.starts_with("EMAIL") {
                        // Parse email (simplified)
                        if let Some(colon_pos) = line.find(':') {
                            let value = line[colon_pos + 1..].to_string();
                            let type_str = if line.contains("TYPE=HOME") {
                                "home"
                            } else if line.contains("TYPE=WORK") {
                                "work"
                            } else {
                                "other"
                            };
                            
                            let is_primary = line.contains("TYPE=PREF");
                            
                            contact.emails.push(ContactEmail {
                                address: value,
                                type_: type_str.to_string(),
                                is_primary,
                            });
                        }
                    } else if line.starts_with("TEL") {
                        // Parse phone (simplified)
                        if let Some(colon_pos) = line.find(':') {
                            let value = line[colon_pos + 1..].to_string();
                            let type_str = if line.contains("TYPE=HOME") {
                                "home"
                            } else if line.contains("TYPE=WORK") {
                                "work"
                            } else if line.contains("TYPE=CELL") {
                                "mobile"
                            } else {
                                "other"
                            };
                            
                            let is_primary = line.contains("TYPE=PREF");
                            
                            contact.phones.push(ContactPhone {
                                number: value,
                                type_: type_str.to_string(),
                                is_primary,
                            });
                        }
                    } else if line.starts_with("ADR") {
                        // Parse address (simplified)
                        if let Some(colon_pos) = line.find(':') {
                            let value = line[colon_pos + 1..].to_string();
                            let parts: Vec<&str> = value.split(';').collect();
                            
                            let type_str = if line.contains("TYPE=HOME") {
                                "home"
                            } else if line.contains("TYPE=WORK") {
                                "work"
                            } else {
                                "other"
                            };
                            
                            let is_primary = line.contains("TYPE=PREF");
                            
                            let mut address = ContactAddress {
                                street: None,
                                city: None,
                                state: None,
                                postal_code: None,
                                country: None,
                                type_: type_str.to_string(),
                                is_primary,
                            };
                            
                            // vCard 3.0 ADR format: PO Box;Extended Address;Street;City;State;Postal Code;Country
                            if parts.len() >= 7 {
                                if !parts[2].is_empty() {
                                    address.street = Some(parts[2].to_string());
                                }
                                if !parts[3].is_empty() {
                                    address.city = Some(parts[3].to_string());
                                }
                                if !parts[4].is_empty() {
                                    address.state = Some(parts[4].to_string());
                                }
                                if !parts[5].is_empty() {
                                    address.postal_code = Some(parts[5].to_string());
                                }
                                if !parts[6].is_empty() {
                                    address.country = Some(parts[6].to_string());
                                }
                            }
                            
                            contact.addresses.push(address);
                        }
                    } else if line.starts_with("ORG:") {
                        // Parse organization (simplified)
                        let value = line[4..].to_string();
                        let parts: Vec<&str> = value.split(';').collect();
                        
                        let mut org = ContactOrganization {
                            name: parts[0].to_string(),
                            title: None,
                            department: None,
                            is_primary: true,
                        };
                        
                        if parts.len() > 1 && !parts[1].is_empty() {
                            org.department = Some(parts[1].to_string());
                        }
                        
                        contact.organizations.push(org);
                    } else if line.starts_with("TITLE:") {
                        // Parse title
                        let title = line[6..].to_string();
                        
                        // Add title to the first organization or create a new one
                        if let Some(org) = contact.organizations.first_mut() {
                            org.title = Some(title);
                        } else {
                            contact.organizations.push(ContactOrganization {
                                name: String::new(),
                                title: Some(title),
                                department: None,
                                is_primary: true,
                            });
                        }
                    } else if line.starts_with("NOTE:") {
                        // Parse note
                        contact.notes = Some(line[5..].to_string());
                    } else if line.starts_with("PHOTO:") || line.starts_with("PHOTO;") {
                        // Parse photo URL (simplified)
                        if let Some(colon_pos) = line.find(':') {
                            contact.photo_url = Some(line[colon_pos + 1..].to_string());
                        }
                    }
                }
            }
        }
        
        Ok(contacts)
    }
    
    /// Create vCard data from a contact
    fn create_vcard(&self, contact: &Contact) -> Result<String> {
        let mut vcard = String::new();
        
        // Add vCard header
        vcard.push_str("BEGIN:VCARD\r\n");
        vcard.push_str(&format!("VERSION:{}\r\n", self.config.vcard_version.version_string()));
        
        // Add UID
        vcard.push_str(&format!("UID:{}\r\n", contact.id));
        
        // Add full name (FN)
        vcard.push_str(&format!("FN:{}\r\n", contact.display_name));
        
        // Add structured name (N)
        let first = contact.first_name.as_deref().unwrap_or("");
        let last = contact.last_name.as_deref().unwrap_or("");
        vcard.push_str(&format!("N:{};{};;;\r\n", last, first));
        
        // Add emails
        for email in &contact.emails {
            let type_param = match email.type_.as_str() {
                "home" => "TYPE=HOME",
                "work" => "TYPE=WORK",
                _ => "TYPE=OTHER",
            };
            
            let pref_param = if email.is_primary { ",PREF" } else { "" };
            
            vcard.push_str(&format!("EMAIL;{}{}:{}\r\n", type_param, pref_param, email.address));
        }
        
        // Add phones
        for phone in &contact.phones {
            let type_param = match phone.type_.as_str() {
                "home" => "TYPE=HOME",
                "work" => "TYPE=WORK",
                "mobile" => "TYPE=CELL",
                _ => "TYPE=OTHER",
            };
            
            let pref_param = if phone.is_primary { ",PREF" } else { "" };
            
            vcard.push_str(&format!("TEL;{}{}:{}\r\n", type_param, pref_param, phone.number));
        }
        
        // Add addresses
        for address in &contact.addresses {
            let type_param = match address.type_.as_str() {
                "home" => "TYPE=HOME",
                "work" => "TYPE=WORK",
                _ => "TYPE=OTHER",
            };
            
            let pref_param = if address.is_primary { ",PREF" } else { "" };
            
            let street = address.street.as_deref().unwrap_or("");
            let city = address.city.as_deref().unwrap_or("");
            let state = address.state.as_deref().unwrap_or("");
            let postal_code = address.postal_code.as_deref().unwrap_or("");
            let country = address.country.as_deref().unwrap_or("");
            
            vcard.push_str(&format!("ADR;{}{};:;;{};{};{};{};{}\r\n", 
                type_param, pref_param, street, city, state, postal_code, country));
        }
        
        // Add organizations
        for (i, org) in contact.organizations.iter().enumerate() {
            vcard.push_str(&format!("ORG:{}", org.name));
            
            if let Some(department) = &org.department {
                vcard.push_str(&format!(";{}", department));
            }
            
            vcard.push_str("\r\n");
            
            if let Some(title) = &org.title {
                vcard.push_str(&format!("TITLE:{}\r\n", title));
            }
            
            // Only add one organization in this simplified implementation
            if i == 0 {
                break;
            }
        }
        
        // Add notes
        if let Some(notes) = &contact.notes {
            vcard.push_str(&format!("NOTE:{}\r\n", notes));
        }
        
        // Add photo URL
        if let Some(photo_url) = &contact.photo_url {
            vcard.push_str(&format!("PHOTO:{}\r\n", photo_url));
        }
        
        // End vCard
        vcard.push_str("END:VCARD\r\n");
        
        Ok(vcard)
    }
}

#[async_trait]
impl WebDavProtocolClient for CardDavClient {
    fn webdav_client(&self) -> &WebDavClient {
        &self.webdav
    }
    
    async fn discover_resources(&self, resource_type: ResourceType) -> Result<Vec<Resource>> {
        // For CardDAV, we're interested in address book collections
        if resource_type != ResourceType::AddressBook {
            return Ok(Vec::new());
        }
        
        // Find the address book home set
        let addressbook_home = self.find_addressbook_home_set().await?;
        
        // Get all collections in the address book home
        let props = vec![
            PropertyName::dav("resourcetype"),
            PropertyName::dav("displayname"),
            PropertyName::carddav("addressbook-description"),
            PropertyName::new("http://apple.com/ns/carddav/", "contacts-count"),
        ];
        
        let resources = self.webdav.propfind(&addressbook_home, Depth::One, &props).await?;
        
        // Filter for address book collections
        let addressbooks = resources.into_iter()
            .filter(|r| r.resource_type == ResourceType::AddressBook)
            .collect();
        
        Ok(addressbooks)
    }
    
    async fn get_resource(&self, url: &str) -> Result<Resource> {
        let props = vec![
            PropertyName::dav("resourcetype"),
            PropertyName::dav("displayname"),
            PropertyName::carddav("addressbook-description"),
            PropertyName::new("http://apple.com/ns/carddav/", "contacts-count"),
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
        
        // Set the resourcetype to address book
        let props = vec![
            Property {
                name: PropertyName::dav("resourcetype"),
                value: PropertyValue::Element(
                    "<D:collection xmlns:D=\"DAV:\"/><C:addressbook xmlns:C=\"urn:ietf:params:xml:ns:carddav\"/>".as_bytes().to_vec()
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
impl crate::integration::interfaces::ContactService for CardDavClient {
    async fn list_contact_groups(&self) -> Result<Vec<ContactGroup>> {
        let resources = self.discover_resources(ResourceType::AddressBook).await?;
        
        let mut groups = Vec::new();
        for resource in resources {
            let group = self.resource_to_contact_group(&resource)?;
            groups.push(group);
        }
        
        Ok(groups)
    }
    
    async fn get_contacts(&self, group_id: Option<&str>) -> Result<Vec<Contact>> {
        // If no group ID is provided, use the default address book
        let addressbook_id = if let Some(id) = group_id {
            id.to_string()
        } else if let Some(default) = &self.config.default_addressbook_url {
            default.clone()
        } else {
            // Try to find the first address book
            let groups = self.list_contact_groups().await?;
            if let Some(first) = groups.first() {
                first.id.clone()
            } else {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "No address book found"
                ));
            }
        };
        
        // Build the address book query
        let query = self.build_addressbook_query()?;
        
        // Send the REPORT request
        let response = self.webdav.report(&addressbook_id, query).await?;
        
        // Parse the response to extract vCard data
        // This is a simplified implementation - in a real implementation,
        // you would need to parse the XML response to extract the address-data elements
        
        // For now, we'll just assume the response contains vCard data
        let contacts = self.parse_vcard(&response)?;
        
        // Set the group ID for all contacts
        let mut contacts_with_group = contacts;
        for contact in &mut contacts_with_group {
            contact.group_ids = vec![addressbook_id.clone()];
        }
        
        Ok(contacts_with_group)
    }
    
    async fn create_contact(&self, contact: &Contact) -> Result<Contact> {
        // Determine which address book to use
        let addressbook_id = if let Some(group_id) = contact.group_ids.first() {
            group_id.clone()
        } else if let Some(default) = &self.config.default_addressbook_url {
            default.clone()
        } else {
            // Try to find the first address book
            let groups = self.list_contact_groups().await?;
            if let Some(first) = groups.first() {
                first.id.clone()
            } else {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "No address book found"
                ));
            }
        };
        
        // Create vCard data
        let vcard_data = self.create_vcard(contact)?;
        
        // Create a URL for the contact
        let contact_url = format!("{}/{}.vcf", addressbook_id, contact.id);
        
        // Put the contact
        self.webdav.put(&contact_url, self.config.vcard_version.content_type(), vcard_data.into_bytes()).await?;
        
        // Return the contact (in a real implementation, you might want to fetch the contact to get server-assigned properties)
        Ok(contact.clone())
    }
    
    async fn update_contact(&self, contact: &Contact) -> Result<Contact> {
        // Determine which address book to use
        let addressbook_id = if let Some(group_id) = contact.group_ids.first() {
            group_id.clone()
        } else {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Contact must have at least one group ID"
            ));
        };
        
        // Create vCard data
        let vcard_data = self.create_vcard(contact)?;
        
        // Create a URL for the contact
        let contact_url = format!("{}/{}.vcf", addressbook_id, contact.id);
        
        // Put the contact
        self.webdav.put(&contact_url, self.config.vcard_version.content_type(), vcard_data.into_bytes()).await?;
        
        // Return the contact (in a real implementation, you might want to fetch the contact to get server-assigned properties)
        Ok(contact.clone())
    }
    
    async fn delete_contact(&self, contact_id: &str) -> Result<()> {
        // In a real implementation, you would need to know which address book the contact is in
        // For simplicity, we'll assume the contact ID includes the address book ID
        
        // Try to find the contact in all address books
        let groups = self.list_contact_groups().await?;
        
        for group in groups {
            // Try to delete the contact from this address book
            let contact_url = format!("{}/{}.vcf", group.id, contact_id);
            
            // Ignore errors - the contact might not be in this address book
            let _ = self.webdav.delete(&contact_url).await;
        }
        
        Ok(())
    }
}