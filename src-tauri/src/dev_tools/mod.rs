//! Developer tools for the application
//!
//! This module provides various tools for developers, including debugging tools,
//! profiling utilities, and documentation generation.

pub mod actor_debug;
pub mod docs_gen;
pub mod profiling;
pub mod adaptive_profiling;
pub mod performance_monitoring;

/// Initialize all developer tools
pub fn init() {
    // Initialize actor debugging tools
    actor_debug::init();

    // Initialize documentation generation tools
    docs_gen::init();

    // Initialize performance profiling tools
    profiling::init();

    // Initialize adaptive performance profiling tools
    adaptive_profiling::init();

    // Initialize performance monitoring and alerting
    performance_monitoring::init();

    tracing::info!("Developer tools initialized");
}
