//! Rolling sphere scenario

use crate::{MetricWorld, BodyBuilder, Scenario};
use std::collections::HashMap;
use super::{get_f32, get_vec3};

/// Scenario: Sphere rolling on a flat surface
pub struct RollingSphereScenario {
    pub radius: f32,
    pub initial_velocity: [f32; 3],
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
}

impl Default for RollingSphereScenario {
    fn default() -> Self {
        Self {
            radius: 0.5,
            initial_velocity: [5.0, 0.0, 0.0],
            friction: 0.5,
            restitution: 0.1,
            density: 1.0,
        }
    }
}

impl RollingSphereScenario {
    pub fn from_params(params: &HashMap<String, serde_yaml::Value>) -> Self {
        Self {
            radius: get_f32(params, "radius", 0.5),
            initial_velocity: get_vec3(params, "initial_velocity", [5.0, 0.0, 0.0]),
            friction: get_f32(params, "friction", 0.5),
            restitution: get_f32(params, "restitution", 0.1),
            density: get_f32(params, "density", 1.0),
        }
    }
}

impl Scenario for RollingSphereScenario {
    fn name(&self) -> &str {
        "rolling_sphere"
    }

    fn description(&self) -> &str {
        "Sphere rolling on a flat surface, tests friction and angular momentum"
    }

    fn setup(&self, world: &mut MetricWorld) {
        // Add ground plane
        let (ground_body, ground_collider, ground_name) = BodyBuilder::new("ground")
            .position_xyz(0.0, -0.5, 0.0)
            .box_shape(100.0, 0.5, 10.0)
            .fixed()
            .friction(self.friction)
            .restitution(self.restitution)
            .build();

        let ground_handle = world.add_body(ground_body, ground_name);
        world.add_collider(ground_collider, ground_handle);

        // Add rolling sphere
        let (sphere_body, sphere_collider, sphere_name) = BodyBuilder::new("sphere")
            .position_xyz(0.0, self.radius, 0.0)
            .velocity_xyz(
                self.initial_velocity[0],
                self.initial_velocity[1],
                self.initial_velocity[2],
            )
            .sphere(self.radius)
            .dynamic()
            .friction(self.friction)
            .restitution(self.restitution)
            .density(self.density)
            .build();

        let sphere_handle = world.add_body(sphere_body, sphere_name);
        world.add_collider(sphere_collider, sphere_handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simuforge_core::PhysicsConfig;

    #[test]
    fn test_rolling_sphere_setup() {
        let config = PhysicsConfig::default();
        let mut world = MetricWorld::new(&config);

        let scenario = RollingSphereScenario::default();
        scenario.setup(&mut world);

        assert_eq!(world.body_count(), 2);
    }
}
