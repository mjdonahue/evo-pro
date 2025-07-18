//! Common protocol implementations for external system integration
//!
//! This module provides implementations for standard protocols like
//! CalDAV and CardDAV to enable integration with external calendar
//! and contact services.

pub mod caldav;
pub mod carddav;
pub mod common;

pub use caldav::*;
pub use carddav::*;
pub use common::*;