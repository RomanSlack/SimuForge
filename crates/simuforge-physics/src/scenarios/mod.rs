//! Built-in simulation scenarios

mod box_stack;
mod rolling;
mod bouncing;
mod friction_ramp;

use crate::{MetricWorld, BodyBuilder};
use simuforge_core::{PhysicsConfig, spec::ScenarioConfig};
use std::collections::HashMap;

pub use box_stack::BoxStackScenario;
pub use rolling::RollingSphereScenario;
pub use bouncing::BouncingBallScenario;
pub use friction_ramp::FrictionRampScenario;

/// Trait for scenario implementations
pub trait Scenario {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn setup(&self, world: &mut MetricWorld);
}

/// Create a scenario from configuration
pub fn create_scenario(config: &ScenarioConfig) -> Box<dyn Scenario> {
    match config {
        ScenarioConfig::Builtin { name, params } => {
            match name.as_str() {
                "box_stack" => Box::new(BoxStackScenario::from_params(params)),
                "rolling_sphere" | "rolling" => Box::new(RollingSphereScenario::from_params(params)),
                "bouncing_ball" | "bouncing" => Box::new(BouncingBallScenario::from_params(params)),
                "friction_ramp" | "ramp" => Box::new(FrictionRampScenario::from_params(params)),
                _ => panic!("Unknown scenario: {}", name),
            }
        }
        ScenarioConfig::Custom { bodies } => {
            Box::new(CustomScenario { bodies: bodies.clone() })
        }
    }
}

/// Custom scenario from body configurations
struct CustomScenario {
    bodies: Vec<simuforge_core::spec::BodyConfig>,
}

impl Scenario for CustomScenario {
    fn name(&self) -> &str {
        "custom"
    }

    fn description(&self) -> &str {
        "Custom user-defined scenario"
    }

    fn setup(&self, world: &mut MetricWorld) {
        for body_config in &self.bodies {
            let (body, collider, name) = BodyBuilder::from_config(body_config).build();
            let handle = world.add_body(body, name);
            world.add_collider(collider, handle);
        }
    }
}

/// Helper to extract f32 from YAML value
pub(crate) fn get_f32(params: &HashMap<String, serde_yaml::Value>, key: &str, default: f32) -> f32 {
    if let Some(value) = params.get(key) {
        if let Some(f) = value.as_f64() {
            return f as f32;
        }
    }
    default
}

/// Helper to extract u32 from YAML value
pub(crate) fn get_u32(params: &HashMap<String, serde_yaml::Value>, key: &str, default: u32) -> u32 {
    if let Some(value) = params.get(key) {
        if let Some(u) = value.as_u64() {
            return u as u32;
        }
    }
    default
}

/// Helper to extract Vec3 from YAML value
pub(crate) fn get_vec3(params: &HashMap<String, serde_yaml::Value>, key: &str, default: [f32; 3]) -> [f32; 3] {
    if let Some(value) = params.get(key) {
        if let Some(seq) = value.as_sequence() {
            let mut arr = default;
            for (i, val) in seq.iter().take(3).enumerate() {
                if let Some(f) = val.as_f64() {
                    arr[i] = f as f32;
                }
            }
            return arr;
        }
    }
    default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_box_stack() {
        let config = ScenarioConfig::Builtin {
            name: "box_stack".to_string(),
            params: HashMap::new(),
        };
        let scenario = create_scenario(&config);
        assert_eq!(scenario.name(), "box_stack");
    }
}
