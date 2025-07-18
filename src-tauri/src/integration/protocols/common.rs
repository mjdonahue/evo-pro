//! Common functionality for WebDAV-based protocols
//!
//! This module provides shared functionality for WebDAV-based protocols
//! like CalDAV and CardDAV, including common operations, XML parsing,
//! and shared data structures.

use async_trait::async_trait;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use quick_xml::{Reader, Writer};
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use crate::error::{Error, ErrorKind, Result};
use crate::integration::auth::{AuthToken, AuthMethod};

/// WebDAV property name with namespace
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyName {
    /// XML namespace
    pub namespace: String,

    /// Property name
    pub name: String,
}

impl PropertyName {
    /// Create a new property name
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }

    /// Create a new DAV property name
    pub fn dav(name: &str) -> Self {
        Self::new("DAV:", name)
    }

    /// Create a new CalDAV property name
    pub fn caldav(name: &str) -> Self {
        Self::new("urn:ietf:params:xml:ns:caldav", name)
    }

    /// Create a new CardDAV property name
    pub fn carddav(name: &str) -> Self {
        Self::new("urn:ietf:params:xml:ns:carddav", name)
    }

    /// Get the qualified name with namespace prefix
    pub fn qualified_name(&self, ns_prefix: &str) -> String {
        format!("{}:{}", ns_prefix, self.name)
    }
}

/// WebDAV property value
#[derive(Debug, Clone)]
pub enum PropertyValue {
    /// Text value
    Text(String),

    /// XML element value
    Element(Vec<u8>),

    /// Multiple values
    Multi(Vec<PropertyValue>),
}

/// WebDAV property
#[derive(Debug, Clone)]
pub struct Property {
    /// Property name
    pub name: PropertyName,

    /// Property value
    pub value: PropertyValue,
}

/// WebDAV resource type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceType {
    /// Regular resource
    Resource,

    /// Collection
    Collection,

    /// Calendar collection
    Calendar,

    /// Address book collection
    AddressBook,
}

/// WebDAV resource
#[derive(Debug, Clone)]
pub struct Resource {
    /// Resource URL
    pub url: String,

    /// Resource type
    pub resource_type: ResourceType,

    /// Resource properties
    pub properties: HashMap<PropertyName, PropertyValue>,

    /// ETag for the resource
    pub etag: Option<String>,

    /// Last modified time
    pub last_modified: Option<String>,

    /// Content type
    pub content_type: Option<String>,

    /// Content length
    pub content_length: Option<u64>,
}

/// WebDAV depth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    /// Current resource only
    Zero,

    /// Current resource and immediate children
    One,

    /// Current resource and all descendants
    Infinity,
}

impl std::fmt::Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Depth::Zero => write!(f, "0"),
            Depth::One => write!(f, "1"),
            Depth::Infinity => write!(f, "infinity"),
        }
    }
}

/// WebDAV client configuration
#[derive(Debug, Clone)]
pub struct WebDavConfig {
    /// Base URL for the WebDAV server
    pub base_url: String,

    /// Authentication method
    pub auth_method: AuthMethod,

    /// Authentication token
    pub auth_token: Option<AuthToken>,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// User agent string
    pub user_agent: String,

    /// Additional headers
    pub headers: HashMap<String, String>,
}

impl Default for WebDavConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            auth_method: AuthMethod::Basic,
            auth_token: None,
            timeout_seconds: 30,
            user_agent: format!("evo-pro-webdav/1.0"),
            headers: HashMap::new(),
        }
    }
}

/// WebDAV client for interacting with WebDAV servers
pub struct WebDavClient {
    /// HTTP client
    client: Client,

    /// WebDAV configuration
    config: WebDavConfig,
}

impl WebDavClient {
    /// Create a new WebDAV client
    pub fn new(config: WebDavConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                Error::new(
                    ErrorKind::Configuration,
                    &format!("Failed to create HTTP client: {}", e)
                )
            })?;

        Ok(Self { client, config })
    }

    /// Create a request builder with authentication
    fn request(&self, method: Method, url: &str) -> RequestBuilder {
        let url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("{}{}", self.config.base_url, url)
        };

        let mut builder = self.client.request(method, url)
            .header("User-Agent", &self.config.user_agent);

        // Add authentication
        if let Some(token) = &self.config.auth_token {
            match self.config.auth_method {
                AuthMethod::Basic => {
                    builder = builder.header("Authorization", format!("Basic {}", token.token));
                },
                AuthMethod::OAuth2 | AuthMethod::OAuth2Pkce => {
                    builder = builder.header("Authorization", format!("Bearer {}", token.token));
                },
                AuthMethod::Jwt => {
                    builder = builder.header("Authorization", format!("Bearer {}", token.token));
                },
                AuthMethod::ApiKey => {
                    // Check if there's a custom header name for the API key
                    let header_name = token.properties.get("header_name").unwrap_or(&"X-API-Key".to_string());
                    builder = builder.header(header_name, &token.token);
                },
                AuthMethod::MultiFactorAuth => {
                    // For MFA, use the token type to determine the header format
                    let header_value = match token.token_type.as_str() {
                        "Bearer" => format!("Bearer {}", token.token),
                        "Basic" => format!("Basic {}", token.token),
                        _ => format!("{} {}", token.token_type, token.token),
                    };

                    builder = builder.header("Authorization", header_value);

                    // Add any MFA-specific headers
                    if let Some(mfa_header) = token.properties.get("mfa_header_name") {
                        if let Some(mfa_value) = token.properties.get("mfa_header_value") {
                            builder = builder.header(mfa_header, mfa_value);
                        }
                    }
                },
                AuthMethod::Custom(ref name) => {
                    // For custom auth methods, check the properties for header information
                    if let Some(header_name) = token.properties.get("header_name") {
                        if let Some(header_format) = token.properties.get("header_format") {
                            let header_value = header_format.replace("{token}", &token.token);
                            builder = builder.header(header_name, header_value);
                        } else {
                            // Default to just using the token as the header value
                            builder = builder.header(header_name, &token.token);
                        }
                    } else {
                        // Default to using the custom name as the header name
                        builder = builder.header(name, &token.token);
                    }
                },
            }

            // Add any additional authentication headers from token properties
            if let Some(additional_headers) = token.properties.get("additional_headers") {
                if let Ok(headers: HashMap<String, String>) = serde_json::from_str(additional_headers) {
                    for (name, value) in headers {
                        builder = builder.header(name, value);
                    }
                }
            }
        }

        // Add custom headers
        for (name, value) in &self.config.headers {
            builder = builder.header(name, value);
        }

        builder
    }

    /// Update the authentication token
    pub fn update_auth_token(&mut self, token: AuthToken) {
        self.config.auth_token = Some(token);
    }

    /// Get the current authentication token
    pub fn auth_token(&self) -> Option<&AuthToken> {
        self.config.auth_token.as_ref()
    }

    /// Set the authentication method
    pub fn set_auth_method(&mut self, method: AuthMethod) {
        self.config.auth_method = method;
    }

    /// Send a PROPFIND request to get properties of a resource
    pub async fn propfind(&self, url: &str, depth: Depth, props: &[PropertyName]) -> Result<Vec<Resource>> {
        // Build the PROPFIND request body
        let body = self.build_propfind_body(props)?;

        // Send the request
        let response = self.request(Method::from_bytes(b"PROPFIND").unwrap(), url)
            .header("Depth", depth.to_string())
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Network,
                    &format!("PROPFIND request failed: {}", e)
                )
            })?;

        // Check the response status
        if response.status() != StatusCode::MULTI_STATUS {
            return Err(Error::new(
                ErrorKind::External,
                &format!("PROPFIND request failed with status: {}", response.status())
            ));
        }

        // Parse the response
        let response_text = response.text().await.map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to read PROPFIND response: {}", e)
            )
        })?;

        self.parse_propfind_response(&response_text)
    }

    /// Build the PROPFIND request body
    fn build_propfind_body(&self, props: &[PropertyName]) -> Result<String> {
        let mut writer = Writer::new(Vec::new());

        // Write the XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write XML declaration: {}", e)
                )
            })?;

        // Start the propfind element
        let mut propfind_elem = BytesStart::new("propfind");
        propfind_elem.push_attribute(("xmlns", "DAV:"));
        writer.write_event(Event::Start(propfind_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write propfind element: {}", e)
                )
            })?;

        // Start the prop element
        let prop_elem = BytesStart::new("prop");
        writer.write_event(Event::Start(prop_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write prop element: {}", e)
                )
            })?;

        // Write the property elements
        for prop in props {
            let mut elem = BytesStart::new(&prop.name);
            if prop.namespace != "DAV:" {
                elem.push_attribute(("xmlns", prop.namespace.as_str()));
            }
            writer.write_event(Event::Empty(elem))
                .map_err(|e| {
                    Error::new(
                        ErrorKind::Internal,
                        &format!("Failed to write property element: {}", e)
                    )
                })?;
        }

        // End the prop element
        writer.write_event(Event::End(BytesEnd::new("prop")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end prop element: {}", e)
                )
            })?;

        // End the propfind element
        writer.write_event(Event::End(BytesEnd::new("propfind")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end propfind element: {}", e)
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

    /// Parse the PROPFIND response
    fn parse_propfind_response(&self, response_text: &str) -> Result<Vec<Resource>> {
        let mut reader = Reader::from_str(response_text);
        reader.trim_text(true);

        let mut resources = Vec::new();
        let mut current_resource: Option<Resource> = None;
        let mut current_property: Option<(PropertyName, Vec<u8>)> = None;
        let mut in_prop = false;
        let mut in_propstat = false;
        let mut current_status = String::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name() {
                        b"response" => {
                            // Start a new resource
                            current_resource = Some(Resource {
                                url: String::new(),
                                resource_type: ResourceType::Resource,
                                properties: HashMap::new(),
                                etag: None,
                                last_modified: None,
                                content_type: None,
                                content_length: None,
                            });
                        },
                        b"href" => {
                            if let Some(resource) = &mut current_resource {
                                // Read the URL
                                if let Ok(Event::Text(e)) = reader.read_event(&mut buf) {
                                    resource.url = String::from_utf8_lossy(&e).to_string();
                                }
                            }
                        },
                        b"propstat" => {
                            in_propstat = true;
                        },
                        b"prop" => {
                            in_prop = true;
                        },
                        b"status" => {
                            if in_propstat {
                                // Read the status
                                if let Ok(Event::Text(e)) = reader.read_event(&mut buf) {
                                    current_status = String::from_utf8_lossy(&e).to_string();
                                }
                            }
                        },
                        b"resourcetype" => {
                            if in_prop && current_status.contains("200") {
                                // Check for collection types
                                let mut is_collection = false;
                                let mut is_calendar = false;
                                let mut is_addressbook = false;

                                loop {
                                    match reader.read_event(&mut buf) {
                                        Ok(Event::Empty(ref e)) => {
                                            match e.name() {
                                                b"collection" => is_collection = true,
                                                b"calendar" => is_calendar = true,
                                                b"addressbook" => is_addressbook = true,
                                                _ => {}
                                            }
                                        },
                                        Ok(Event::End(ref e)) if e.name() == b"resourcetype" => break,
                                        Ok(Event::Eof) => break,
                                        _ => {}
                                    }
                                }

                                if let Some(resource) = &mut current_resource {
                                    if is_calendar {
                                        resource.resource_type = ResourceType::Calendar;
                                    } else if is_addressbook {
                                        resource.resource_type = ResourceType::AddressBook;
                                    } else if is_collection {
                                        resource.resource_type = ResourceType::Collection;
                                    }
                                }
                            }
                        },
                        b"getetag" => {
                            if in_prop && current_status.contains("200") {
                                // Read the ETag
                                if let Ok(Event::Text(e)) = reader.read_event(&mut buf) {
                                    if let Some(resource) = &mut current_resource {
                                        resource.etag = Some(String::from_utf8_lossy(&e).to_string());
                                    }
                                }
                            }
                        },
                        b"getlastmodified" => {
                            if in_prop && current_status.contains("200") {
                                // Read the last modified time
                                if let Ok(Event::Text(e)) = reader.read_event(&mut buf) {
                                    if let Some(resource) = &mut current_resource {
                                        resource.last_modified = Some(String::from_utf8_lossy(&e).to_string());
                                    }
                                }
                            }
                        },
                        b"getcontenttype" => {
                            if in_prop && current_status.contains("200") {
                                // Read the content type
                                if let Ok(Event::Text(e)) = reader.read_event(&mut buf) {
                                    if let Some(resource) = &mut current_resource {
                                        resource.content_type = Some(String::from_utf8_lossy(&e).to_string());
                                    }
                                }
                            }
                        },
                        b"getcontentlength" => {
                            if in_prop && current_status.contains("200") {
                                // Read the content length
                                if let Ok(Event::Text(e)) = reader.read_event(&mut buf) {
                                    if let Some(resource) = &mut current_resource {
                                        let length_str = String::from_utf8_lossy(&e).to_string();
                                        if let Ok(length) = length_str.parse::<u64>() {
                                            resource.content_length = Some(length);
                                        }
                                    }
                                }
                            }
                        },
                        _ => {
                            if in_prop && current_status.contains("200") {
                                // Start a new property
                                let name_str = String::from_utf8_lossy(e.name()).to_string();
                                let namespace = if let Some(ns) = e.attributes()
                                    .find_map(|a| a.ok())
                                    .filter(|a| a.key == b"xmlns")
                                    .map(|a| String::from_utf8_lossy(&a.value).to_string()) {
                                    ns
                                } else {
                                    "DAV:".to_string()
                                };

                                current_property = Some((
                                    PropertyName {
                                        namespace,
                                        name: name_str,
                                    },
                                    Vec::new(),
                                ));
                            }
                        }
                    }
                },
                Ok(Event::Text(e)) => {
                    if let Some((_, ref mut value)) = &mut current_property {
                        value.extend_from_slice(&e);
                    }
                },
                Ok(Event::End(ref e)) => {
                    match e.name() {
                        b"response" => {
                            // Add the resource to the list
                            if let Some(resource) = current_resource.take() {
                                resources.push(resource);
                            }
                        },
                        b"propstat" => {
                            in_propstat = false;
                            current_status.clear();
                        },
                        b"prop" => {
                            in_prop = false;
                        },
                        _ => {
                            if in_prop && current_status.contains("200") {
                                // End the current property
                                if let Some((name, value)) = current_property.take() {
                                    if let Some(resource) = &mut current_resource {
                                        resource.properties.insert(
                                            name,
                                            PropertyValue::Text(String::from_utf8_lossy(&value).to_string()),
                                        );
                                    }
                                }
                            }
                        }
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(Error::new(
                        ErrorKind::Parse,
                        &format!("Error parsing XML: {}", e)
                    ));
                },
                _ => {}
            }

            buf.clear();
        }

        Ok(resources)
    }

    /// Send a PROPPATCH request to set properties of a resource
    pub async fn proppatch(&self, url: &str, props: &[Property]) -> Result<()> {
        // Build the PROPPATCH request body
        let body = self.build_proppatch_body(props)?;

        // Send the request
        let response = self.request(Method::from_bytes(b"PROPPATCH").unwrap(), url)
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Network,
                    &format!("PROPPATCH request failed: {}", e)
                )
            })?;

        // Check the response status
        if response.status() != StatusCode::MULTI_STATUS {
            return Err(Error::new(
                ErrorKind::External,
                &format!("PROPPATCH request failed with status: {}", response.status())
            ));
        }

        Ok(())
    }

    /// Build the PROPPATCH request body
    fn build_proppatch_body(&self, props: &[Property]) -> Result<String> {
        let mut writer = Writer::new(Vec::new());

        // Write the XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write XML declaration: {}", e)
                )
            })?;

        // Start the propertyupdate element
        let mut propertyupdate_elem = BytesStart::new("propertyupdate");
        propertyupdate_elem.push_attribute(("xmlns", "DAV:"));
        writer.write_event(Event::Start(propertyupdate_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write propertyupdate element: {}", e)
                )
            })?;

        // Start the set element
        let set_elem = BytesStart::new("set");
        writer.write_event(Event::Start(set_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write set element: {}", e)
                )
            })?;

        // Start the prop element
        let prop_elem = BytesStart::new("prop");
        writer.write_event(Event::Start(prop_elem))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to write prop element: {}", e)
                )
            })?;

        // Write the property elements
        for prop in props {
            let mut elem = BytesStart::new(&prop.name.name);
            if prop.name.namespace != "DAV:" {
                elem.push_attribute(("xmlns", prop.name.namespace.as_str()));
            }

            match &prop.value {
                PropertyValue::Text(text) => {
                    writer.write_event(Event::Start(elem))
                        .map_err(|e| {
                            Error::new(
                                ErrorKind::Internal,
                                &format!("Failed to write property element: {}", e)
                            )
                        })?;

                    writer.write_event(Event::Text(BytesText::from_plain_str(text)))
                        .map_err(|e| {
                            Error::new(
                                ErrorKind::Internal,
                                &format!("Failed to write property text: {}", e)
                            )
                        })?;

                    writer.write_event(Event::End(BytesEnd::new(&prop.name.name)))
                        .map_err(|e| {
                            Error::new(
                                ErrorKind::Internal,
                                &format!("Failed to end property element: {}", e)
                            )
                        })?;
                },
                PropertyValue::Element(xml) => {
                    // For XML elements, we just write the raw XML
                    writer.write_event(Event::Start(elem))
                        .map_err(|e| {
                            Error::new(
                                ErrorKind::Internal,
                                &format!("Failed to write property element: {}", e)
                            )
                        })?;

                    // This is a simplification - in a real implementation, you'd need to parse and rewrite the XML
                    writer.write(xml)
                        .map_err(|e| {
                            Error::new(
                                ErrorKind::Internal,
                                &format!("Failed to write property XML: {}", e)
                            )
                        })?;

                    writer.write_event(Event::End(BytesEnd::new(&prop.name.name)))
                        .map_err(|e| {
                            Error::new(
                                ErrorKind::Internal,
                                &format!("Failed to end property element: {}", e)
                            )
                        })?;
                },
                PropertyValue::Multi(_) => {
                    // Multi-value properties not supported in this simplified implementation
                    return Err(Error::new(
                        ErrorKind::NotImplemented,
                        "Multi-value properties not supported"
                    ));
                }
            }
        }

        // End the prop element
        writer.write_event(Event::End(BytesEnd::new("prop")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end prop element: {}", e)
                )
            })?;

        // End the set element
        writer.write_event(Event::End(BytesEnd::new("set")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end set element: {}", e)
                )
            })?;

        // End the propertyupdate element
        writer.write_event(Event::End(BytesEnd::new("propertyupdate")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to end propertyupdate element: {}", e)
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

    /// Send a GET request to retrieve a resource
    pub async fn get(&self, url: &str) -> Result<Response> {
        let response = self.request(Method::GET, url)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Network,
                    &format!("GET request failed: {}", e)
                )
            })?;

        // Check the response status
        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::External,
                &format!("GET request failed with status: {}", response.status())
            ));
        }

        Ok(response)
    }

    /// Send a PUT request to create or update a resource
    pub async fn put(&self, url: &str, content_type: &str, data: Vec<u8>) -> Result<()> {
        let response = self.request(Method::PUT, url)
            .header("Content-Type", content_type)
            .body(data)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Network,
                    &format!("PUT request failed: {}", e)
                )
            })?;

        // Check the response status
        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::External,
                &format!("PUT request failed with status: {}", response.status())
            ));
        }

        Ok(())
    }

    /// Send a DELETE request to delete a resource
    pub async fn delete(&self, url: &str) -> Result<()> {
        let response = self.request(Method::DELETE, url)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Network,
                    &format!("DELETE request failed: {}", e)
                )
            })?;

        // Check the response status
        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::External,
                &format!("DELETE request failed with status: {}", response.status())
            ));
        }

        Ok(())
    }

    /// Send a MKCOL request to create a collection
    pub async fn mkcol(&self, url: &str) -> Result<()> {
        let response = self.request(Method::from_bytes(b"MKCOL").unwrap(), url)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Network,
                    &format!("MKCOL request failed: {}", e)
                )
            })?;

        // Check the response status
        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::External,
                &format!("MKCOL request failed with status: {}", response.status())
            ));
        }

        Ok(())
    }

    /// Send a REPORT request
    pub async fn report(&self, url: &str, body: String) -> Result<String> {
        let response = self.request(Method::from_bytes(b"REPORT").unwrap(), url)
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Network,
                    &format!("REPORT request failed: {}", e)
                )
            })?;

        // Check the response status
        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::External,
                &format!("REPORT request failed with status: {}", response.status())
            ));
        }

        // Get the response body
        let response_text = response.text().await.map_err(|e| {
            Error::new(
                ErrorKind::Parse,
                &format!("Failed to read REPORT response: {}", e)
            )
        })?;

        Ok(response_text)
    }
}

/// Trait for WebDAV-based protocol clients
#[async_trait]
pub trait WebDavProtocolClient: Send + Sync {
    /// Get the WebDAV client
    fn webdav_client(&self) -> &WebDavClient;

    /// Discover resources of a specific type
    async fn discover_resources(&self, resource_type: ResourceType) -> Result<Vec<Resource>>;

    /// Get a resource by URL
    async fn get_resource(&self, url: &str) -> Result<Resource>;

    /// Create a collection
    async fn create_collection(&self, url: &str) -> Result<()>;

    /// Delete a resource
    async fn delete_resource(&self, url: &str) -> Result<()>;
}
