//! SimuForge Core - Shared types and traits for physics simulation harness
//!
//! This crate provides physics-agnostic types used across the SimuForge ecosystem.

pub mod math;
pub mod spec;
pub mod metrics;
pub mod report;
pub mod error;

pub use math::{Vec3, Quat, Transform};
pub use spec::{ExperimentSpec, PhysicsConfig, DurationConfig, ScenarioConfig, MetricsConfig, CriteriaConfig};
pub use metrics::{MetricFrame, AggregateMetrics, ContactMetrics, EnergyMetrics, MomentumMetrics};
pub use report::{SimulationReport, CriterionResult, BaselineComparison, ReportStatus};
pub use error::SimuForgeError;
