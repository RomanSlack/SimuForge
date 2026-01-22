//! Friction ramp scenario

use crate::{MetricWorld, BodyBuilder, Scenario};
use simuforge_core::Vec3;
use std::collections::HashMap;
use super::get_f32;

/// Scenario: Object sliding down an inclined ramp
pub struct FrictionRampScenario {
    pub ramp_angle: f32,       // Radians
    pub ramp_length: f32,
    pub box_size: f32,
    pub friction: f32,
    pub restitution: f32,
}

impl Default for FrictionRampScenario {
    fn default() -> Self {
        Self {
            ramp_angle: 0.5,      // ~28.6 degrees
            ramp_length: 10.0,
            box_size: 1.0,
            friction: 0.3,
            restitution: 0.1,
        }
    }
}

impl FrictionRampScenario {
    pub fn from_params(params: &HashMap<String, serde_yaml::Value>) -> Self {
        Self {
            ramp_angle: get_f32(params, "ramp_angle", 0.5),
            ramp_length: get_f32(params, "ramp_length", 10.0),
            box_size: get_f32(params, "box_size", 1.0),
            friction: get_f32(params, "friction", 0.3),
            restitution: get_f32(params, "restitution", 0.1),
        }
    }
}

impl Scenario for FrictionRampScenario {
    fn name(&self) -> &str {
        "friction_ramp"
    }

    fn description(&self) -> &str {
        "Object sliding down a ramp, tests friction coefficient accuracy"
    }

    fn setup(&self, world: &mut MetricWorld) {
        let ramp_height = (self.ramp_angle.sin() * self.ramp_length) / 2.0;
        let ramp_offset = (self.ramp_angle.cos() * self.ramp_length) / 2.0;

        // Add ramp (rotated box)
        let (ramp_body, ramp_collider, ramp_name) = BodyBuilder::new("ramp")
            .position_xyz(ramp_offset, ramp_height, 0.0)
            .rotation(Vec3::new(0.0, 0.0, 1.0), -self.ramp_angle)
            .box_shape(self.ramp_length / 2.0, 0.5, 2.0)
            .fixed()
            .friction(self.friction)
            .restitution(self.restitution)
            .build();

        let ramp_handle = world.add_body(ramp_body, ramp_name);
        world.add_collider(ramp_collider, ramp_handle);

        // Add floor at bottom
        let (floor_body, floor_collider, floor_name) = BodyBuilder::new("floor")
            .position_xyz(ramp_offset * 2.0 + 5.0, -0.5, 0.0)
            .box_shape(20.0, 0.5, 5.0)
            .fixed()
            .friction(self.friction)
            .restitution(self.restitution)
            .build();

        let floor_handle = world.add_body(floor_body, floor_name);
        world.add_collider(floor_collider, floor_handle);

        // Add sliding box at top of ramp
        let box_half = self.box_size / 2.0;
        let start_x = self.ramp_angle.cos() * (self.ramp_length * 0.9);
        let start_y = self.ramp_angle.sin() * (self.ramp_length * 0.9) + box_half + 0.6;

        let (box_body, box_collider, box_name) = BodyBuilder::new("slider")
            .position_xyz(start_x, start_y, 0.0)
            .box_shape(box_half, box_half, box_half)
            .dynamic()
            .friction(self.friction)
            .restitution(self.restitution)
            .build();

        let box_handle = world.add_body(box_body, box_name);
        world.add_collider(box_collider, box_handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simuforge_core::PhysicsConfig;

    #[test]
    fn test_friction_ramp_setup() {
        let config = PhysicsConfig::default();
        let mut world = MetricWorld::new(&config);

        let scenario = FrictionRampScenario::default();
        scenario.setup(&mut world);

        assert_eq!(world.body_count(), 3);
    }
}
