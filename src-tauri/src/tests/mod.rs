//! Test modules for the application

#[cfg(test)]
mod error_tests;

#[cfg(test)]
mod error_taxonomy_tests;

#[cfg(test)]
mod error_enrichment_tests;

#[cfg(test)]
mod error_correlation_tests;

#[cfg(test)]
mod logging_tests;

#[cfg(test)]
mod metrics_tests;

#[cfg(test)]
mod supervision_tests;

#[cfg(test)]
mod ipc_tests;

#[cfg(test)]
mod lifecycle_tests;

#[cfg(test)]
pub mod framework;

#[cfg(test)]
mod agents;

#[cfg(test)]
mod actor_tests;
