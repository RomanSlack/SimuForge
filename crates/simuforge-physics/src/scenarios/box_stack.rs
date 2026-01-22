//! Box stacking scenario

use crate::{MetricWorld, BodyBuilder, Scenario};
use rapier3d::prelude::*;
use std::collections::HashMap;
use super::{get_f32, get_u32, get_vec3};

/// Scenario: Stack of boxes on a ground plane
pub struct BoxStackScenario {
    pub count: u32,
    pub box_size: [f32; 3],
    pub spacing: f32,
    pub friction: f32,
    pub restitution: f32,
}

impl Default for BoxStackScenario {
    fn default() -> Self {
        Self {
            count: 10,
            box_size: [1.0, 1.0, 1.0],
            spacing: 0.0,
            friction: 0.5,
            restitution: 0.1,
        }
    }
}

impl BoxStackScenario {
    pub fn new(count: u32) -> Self {
        Self {
            count,
            ..Default::default()
        }
    }

    pub fn from_params(params: &HashMap<String, serde_yaml::Value>) -> Self {
        Self {
            count: get_u32(params, "count", 10),
            box_size: get_vec3(params, "box_size", [1.0, 1.0, 1.0]),
            spacing: get_f32(params, "spacing", 0.0),
            friction: get_f32(params, "friction", 0.5),
            restitution: get_f32(params, "restitution", 0.1),
        }
    }

    pub fn with_box_size(mut self, x: f32, y: f32, z: f32) -> Self {
        self.box_size = [x, y, z];
        self
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }
}

impl Scenario for BoxStackScenario {
    fn name(&self) -> &str {
        "box_stack"
    }

    fn description(&self) -> &str {
        "Stack of boxes on a ground plane, tests stability and contact handling"
    }

    fn setup(&self, world: &mut MetricWorld) {
        let half_x = self.box_size[0] / 2.0;
        let half_y = self.box_size[1] / 2.0;
        let half_z = self.box_size[2] / 2.0;

        // Add ground plane
        let (ground_body, ground_collider, ground_name) = BodyBuilder::new("ground")
            .position_xyz(0.0, -0.5, 0.0)
            .box_shape(50.0, 0.5, 50.0)
            .fixed()
            .friction(self.friction)
            .restitution(self.restitution)
            .build();

        let ground_handle = world.add_body(ground_body, ground_name);
        world.add_collider(ground_collider, ground_handle);

        // Add stacked boxes
        for i in 0..self.count {
            let y = half_y + (self.box_size[1] + self.spacing) * i as f32;
            let name = format!("box_{}", i);

            let (body, collider, name) = BodyBuilder::new(name)
                .position_xyz(0.0, y, 0.0)
                .box_shape(half_x, half_y, half_z)
                .dynamic()
                .friction(self.friction)
                .restitution(self.restitution)
                .build();

            let handle = world.add_body(body, name);
            world.add_collider(collider, handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simuforge_core::PhysicsConfig;

    #[test]
    fn test_box_stack_setup() {
        let config = PhysicsConfig::default();
        let mut world = MetricWorld::new(&config);

        let scenario = BoxStackScenario::new(5);
        scenario.setup(&mut world);

        // Ground + 5 boxes
        assert_eq!(world.body_count(), 6);
    }
}
