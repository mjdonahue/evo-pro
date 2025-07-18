//! Correlation ID management for structured logging
//!
//! This module provides utilities for generating, storing, and retrieving
//! correlation IDs for structured logging. Correlation IDs are used to link
//! related log entries and errors together, making it easier to trace the
//! flow of execution through the system.

use std::cell::RefCell;
use uuid::Uuid;
use tracing::{info, debug};

thread_local! {
    /// Thread-local storage for the current correlation ID
    static CURRENT_CORRELATION_ID: RefCell<Option<String>> = RefCell::new(None);
}

/// Generate a new correlation ID
pub fn generate_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

/// Set the current correlation ID for the current thread
pub fn set_correlation_id(correlation_id: impl Into<String>) {
    let correlation_id = correlation_id.into();
    debug!("Setting correlation ID: {}", correlation_id);
    CURRENT_CORRELATION_ID.with(|current| {
        *current.borrow_mut() = Some(correlation_id);
    });
}

/// Get the current correlation ID for the current thread
pub fn get_correlation_id() -> Option<String> {
    CURRENT_CORRELATION_ID.with(|current| current.borrow().clone())
}

/// Clear the current correlation ID for the current thread
pub fn clear_correlation_id() {
    CURRENT_CORRELATION_ID.with(|current| {
        *current.borrow_mut() = None;
    });
}

/// Execute a function with a specific correlation ID
pub fn with_correlation_id<F, R>(correlation_id: impl Into<String>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let correlation_id = correlation_id.into();
    let previous = get_correlation_id();
    set_correlation_id(correlation_id);
    let result = f();
    match previous {
        Some(id) => set_correlation_id(id),
        None => clear_correlation_id(),
    }
    result
}

/// Execute a function with a new correlation ID
pub fn with_new_correlation_id<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    with_correlation_id(generate_correlation_id(), f)
}

/// Get the current correlation ID or generate a new one if none exists
pub fn get_or_generate_correlation_id() -> String {
    get_correlation_id().unwrap_or_else(generate_correlation_id)
}

/// Create a child correlation ID from a parent correlation ID
pub fn create_child_correlation_id(parent_id: &str) -> String {
    format!("{}.{}", parent_id, Uuid::new_v4().to_string().split('-').next().unwrap_or("child"))
}

/// Execute a function with a child correlation ID
pub fn with_child_correlation_id<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let parent_id = get_or_generate_correlation_id();
    let child_id = create_child_correlation_id(&parent_id);
    with_correlation_id(child_id, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_generation() {
        let id = generate_correlation_id();
        assert!(!id.is_empty());
        assert_ne!(id, generate_correlation_id());
    }

    #[test]
    fn test_set_and_get_correlation_id() {
        let id = generate_correlation_id();
        set_correlation_id(&id);
        assert_eq!(get_correlation_id(), Some(id));
    }

    #[test]
    fn test_clear_correlation_id() {
        let id = generate_correlation_id();
        set_correlation_id(&id);
        assert_eq!(get_correlation_id(), Some(id));
        clear_correlation_id();
        assert_eq!(get_correlation_id(), None);
    }

    #[test]
    fn test_with_correlation_id() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();
        
        set_correlation_id(&id1);
        assert_eq!(get_correlation_id(), Some(id1.clone()));
        
        let result = with_correlation_id(&id2, || {
            assert_eq!(get_correlation_id(), Some(id2.clone()));
            "test"
        });
        
        assert_eq!(result, "test");
        assert_eq!(get_correlation_id(), Some(id1));
    }

    #[test]
    fn test_with_new_correlation_id() {
        let id = generate_correlation_id();
        set_correlation_id(&id);
        assert_eq!(get_correlation_id(), Some(id.clone()));
        
        let result = with_new_correlation_id(|| {
            assert_ne!(get_correlation_id(), Some(id.clone()));
            assert!(get_correlation_id().is_some());
            "test"
        });
        
        assert_eq!(result, "test");
        assert_eq!(get_correlation_id(), Some(id));
    }

    #[test]
    fn test_get_or_generate_correlation_id() {
        clear_correlation_id();
        let id1 = get_or_generate_correlation_id();
        assert!(!id1.is_empty());
        
        set_correlation_id("test-id");
        let id2 = get_or_generate_correlation_id();
        assert_eq!(id2, "test-id");
    }

    #[test]
    fn test_create_child_correlation_id() {
        let parent = "parent-id";
        let child = create_child_correlation_id(parent);
        assert!(child.starts_with("parent-id."));
    }

    #[test]
    fn test_with_child_correlation_id() {
        let id = "parent-id";
        set_correlation_id(id);
        
        let result = with_child_correlation_id(|| {
            let current = get_correlation_id().unwrap();
            assert!(current.starts_with("parent-id."));
            "test"
        });
        
        assert_eq!(result, "test");
        assert_eq!(get_correlation_id(), Some(id.to_string()));
    }
}