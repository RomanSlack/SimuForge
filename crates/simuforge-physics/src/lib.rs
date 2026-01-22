//! SimuForge Physics - Rapier integration with metric collection
//!
//! This crate provides a physics world wrapper that collects metrics
//! during simulation for analysis and comparison.

pub mod world;
pub mod metrics;
pub mod scenarios;
mod body_builder;

pub use world::MetricWorld;
pub use body_builder::BodyBuilder;
pub use scenarios::{Scenario, create_scenario};
