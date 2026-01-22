//! Experiment specification types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::Vec3;

/// Root experiment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentSpec {
    pub api_version: String,
    pub kind: String,
    pub metadata: ExperimentMetadata,
    pub spec: ExperimentConfig,
}

impl ExperimentSpec {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.api_version != "simuforge/v1" {
            errors.push(format!("Unsupported API version: {}", self.api_version));
        }

        if self.kind != "Experiment" {
            errors.push(format!("Unsupported kind: {}", self.kind));
        }

        if self.metadata.name.is_empty() {
            errors.push("Experiment name cannot be empty".to_string());
        }

        if self.spec.physics.timestep <= 0.0 {
            errors.push("Timestep must be positive".to_string());
        }

        if self.spec.physics.solver_iterations == 0 {
            errors.push("Solver iterations must be at least 1".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Experiment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetadata {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Main experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub physics: PhysicsConfig,
    pub duration: DurationConfig,
    pub scenario: ScenarioConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub criteria: HashMap<String, CriteriaConfig>,
}

/// Physics engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PhysicsConfig {
    #[serde(default = "default_timestep")]
    pub timestep: f32,
    #[serde(default = "default_gravity")]
    pub gravity: Vec3,
    #[serde(default = "default_solver_iterations")]
    pub solver_iterations: u32,
    #[serde(default = "default_enhanced_determinism")]
    pub enhanced_determinism: bool,
    #[serde(default)]
    pub seed: Option<u64>,
}

fn default_timestep() -> f32 { 1.0 / 60.0 }
fn default_gravity() -> Vec3 { Vec3::new(0.0, -9.81, 0.0) }
fn default_solver_iterations() -> u32 { 8 }
fn default_enhanced_determinism() -> bool { true }

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            timestep: default_timestep(),
            gravity: default_gravity(),
            solver_iterations: default_solver_iterations(),
            enhanced_determinism: default_enhanced_determinism(),
            seed: None,
        }
    }
}

/// Simulation duration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DurationConfig {
    Fixed { steps: u64 },
    Time { seconds: f32 },
    UntilStable { max_steps: u64, threshold: f32 },
}

impl Default for DurationConfig {
    fn default() -> Self {
        Self::Fixed { steps: 600 }
    }
}

/// Scenario configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScenarioConfig {
    Builtin {
        name: String,
        #[serde(default)]
        params: HashMap<String, serde_yaml::Value>,
    },
    Custom {
        bodies: Vec<BodyConfig>,
    },
}

/// Body configuration for custom scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyConfig {
    pub name: String,
    pub shape: ShapeConfig,
    pub position: Vec3,
    #[serde(default)]
    pub rotation: Option<[f32; 4]>,
    #[serde(default)]
    pub velocity: Option<Vec3>,
    #[serde(default)]
    pub angular_velocity: Option<Vec3>,
    #[serde(default = "default_body_type")]
    pub body_type: BodyType,
    #[serde(default)]
    pub material: MaterialConfig,
}

fn default_body_type() -> BodyType { BodyType::Dynamic }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    Dynamic,
    Static,
    Kinematic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShapeConfig {
    Box { half_extents: Vec3 },
    Sphere { radius: f32 },
    Capsule { half_height: f32, radius: f32 },
    Cylinder { half_height: f32, radius: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialConfig {
    #[serde(default = "default_friction")]
    pub friction: f32,
    #[serde(default = "default_restitution")]
    pub restitution: f32,
    #[serde(default = "default_density")]
    pub density: f32,
}

fn default_friction() -> f32 { 0.5 }
fn default_restitution() -> f32 { 0.3 }
fn default_density() -> f32 { 1.0 }

impl Default for MaterialConfig {
    fn default() -> Self {
        Self {
            friction: default_friction(),
            restitution: default_restitution(),
            density: default_density(),
        }
    }
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsConfig {
    #[serde(default)]
    pub per_frame: Vec<String>,
    #[serde(default)]
    pub aggregate: Vec<String>,
}

/// Pass/fail criteria configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriteriaConfig {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub equals: Option<f64>,
    #[serde(default)]
    pub tolerance: Option<f64>,
}

impl CriteriaConfig {
    pub fn evaluate(&self, value: f64) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        if let Some(equals) = self.equals {
            let tol = self.tolerance.unwrap_or(1e-6);
            if (value - equals).abs() > tol {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_experiment_spec() {
        let yaml = r#"
apiVersion: simuforge/v1
kind: Experiment
metadata:
  name: test-experiment
spec:
  physics:
    timestep: 0.016666667
    gravity: [0, -9.81, 0]
  duration:
    type: fixed
    steps: 100
  scenario:
    type: builtin
    name: box_stack
    params:
      count: 5
"#;
        let spec: ExperimentSpec = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(spec.metadata.name, "test-experiment");
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_criteria_evaluation() {
        let criteria = CriteriaConfig {
            min: Some(0.0),
            max: Some(5.0),
            equals: None,
            tolerance: None,
        };
        assert!(criteria.evaluate(2.5));
        assert!(!criteria.evaluate(6.0));
        assert!(!criteria.evaluate(-1.0));
    }
}
