//! Validation utilities for repositories
//!
//! This module provides validation utilities for repositories, including
//! validation rules and validation error handling.

use std::collections::HashMap;

use crate::entities::tasks::{Task, TaskStatus};
use crate::error::{AppError, Result};

/// Validation rule for a field
pub trait ValidationRule<T> {
    /// Validate the field
    fn validate(&self, value: &T) -> Result<()>;
    
    /// Get the error message for this rule
    fn error_message(&self) -> String;
}

/// Required field validation rule
pub struct Required;

impl<T> ValidationRule<Option<T>> for Required {
    fn validate(&self, value: &Option<T>) -> Result<()> {
        if value.is_none() {
            return Err(AppError::validation(self.error_message()));
        }
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "This field is required".to_string()
    }
}

/// String length validation rule
pub struct StringLength {
    /// Minimum length (inclusive)
    pub min: Option<usize>,
    /// Maximum length (inclusive)
    pub max: Option<usize>,
}

impl ValidationRule<String> for StringLength {
    fn validate(&self, value: &String) -> Result<()> {
        if let Some(min) = self.min {
            if value.len() < min {
                return Err(AppError::validation(self.error_message()));
            }
        }
        
        if let Some(max) = self.max {
            if value.len() > max {
                return Err(AppError::validation(self.error_message()));
            }
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) => format!("Length must be between {} and {} characters", min, max),
            (Some(min), None) => format!("Length must be at least {} characters", min),
            (None, Some(max)) => format!("Length must be at most {} characters", max),
            (None, None) => "Invalid string length".to_string(),
        }
    }
}

/// Date validation rule
pub struct DateValidation {
    /// Whether the date must be in the future
    pub future: bool,
    /// Whether the date must be in the past
    pub past: bool,
}

impl ValidationRule<chrono::DateTime<chrono::Utc>> for DateValidation {
    fn validate(&self, value: &chrono::DateTime<chrono::Utc>) -> Result<()> {
        let now = chrono::Utc::now();
        
        if self.future && *value < now {
            return Err(AppError::validation(self.error_message()));
        }
        
        if self.past && *value > now {
            return Err(AppError::validation(self.error_message()));
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        if self.future {
            "Date must be in the future".to_string()
        } else if self.past {
            "Date must be in the past".to_string()
        } else {
            "Invalid date".to_string()
        }
    }
}

/// Task validator
pub struct TaskValidator;

impl TaskValidator {
    /// Validate a task
    pub fn validate(task: &Task) -> Result<()> {
        let mut errors = HashMap::new();
        
        // Validate title
        if task.title.is_empty() {
            errors.insert("title", "Title is required".to_string());
        } else if task.title.len() > 255 {
            errors.insert("title", "Title must be at most 255 characters".to_string());
        }
        
        // Validate description
        if let Some(desc) = &task.description {
            if desc.len() > 1000 {
                errors.insert("description", "Description must be at most 1000 characters".to_string());
            }
        }
        
        // Validate dates
        if let Some(due_date) = task.due_date {
            if task.start_time > due_date {
                errors.insert("due_date", "Due date must be after start time".to_string());
            }
        }
        
        if let Some(end_time) = task.end_time {
            if task.start_time > end_time {
                errors.insert("end_time", "End time must be after start time".to_string());
            }
            
            // If task is completed, end time must be set
            if task.status == TaskStatus::Completed && task.end_time.is_none() {
                errors.insert("end_time", "End time is required for completed tasks".to_string());
            }
        }
        
        // If there are any errors, return a validation error
        if !errors.is_empty() {
            let error_message = errors.values().cloned().collect::<Vec<String>>().join(", ");
            return Err(AppError::validation(error_message));
        }
        
        Ok(())
    }
}

/// Validation extension trait for repositories
pub trait ValidationExt<T> {
    /// Validate an entity before creating or updating it
    fn validate(&self, entity: &T) -> Result<()>;
}

/// Implement validation for Task
impl ValidationExt<Task> for crate::repositories::TaskRepository {
    fn validate(&self, task: &Task) -> Result<()> {
        TaskValidator::validate(task)
    }
}