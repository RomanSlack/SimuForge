//! Bouncing ball scenario

use crate::{MetricWorld, BodyBuilder, Scenario};
use std::collections::HashMap;
use super::get_f32;

/// Scenario: Ball dropped from height, bouncing on ground
pub struct BouncingBallScenario {
    pub radius: f32,
    pub drop_height: f32,
    pub restitution: f32,
    pub friction: f32,
    pub density: f32,
}

impl Default for BouncingBallScenario {
    fn default() -> Self {
        Self {
            radius: 0.5,
            drop_height: 10.0,
            restitution: 0.8,
            friction: 0.3,
            density: 1.0,
        }
    }
}

impl BouncingBallScenario {
    pub fn from_params(params: &HashMap<String, serde_yaml::Value>) -> Self {
        Self {
            radius: get_f32(params, "radius", 0.5),
            drop_height: get_f32(params, "drop_height", 10.0),
            restitution: get_f32(params, "restitution", 0.8),
            friction: get_f32(params, "friction", 0.3),
            density: get_f32(params, "density", 1.0),
        }
    }
}

impl Scenario for BouncingBallScenario {
    fn name(&self) -> &str {
        "bouncing_ball"
    }

    fn description(&self) -> &str {
        "Ball dropped from height, tests restitution and energy conservation"
    }

    fn setup(&self, world: &mut MetricWorld) {
        // Add ground plane
        let (ground_body, ground_collider, ground_name) = BodyBuilder::new("ground")
            .position_xyz(0.0, -0.5, 0.0)
            .box_shape(10.0, 0.5, 10.0)
            .fixed()
            .friction(self.friction)
            .restitution(self.restitution)
            .build();

        let ground_handle = world.add_body(ground_body, ground_name);
        world.add_collider(ground_collider, ground_handle);

        // Add bouncing ball
        let (ball_body, ball_collider, ball_name) = BodyBuilder::new("ball")
            .position_xyz(0.0, self.drop_height + self.radius, 0.0)
            .sphere(self.radius)
            .dynamic()
            .friction(self.friction)
            .restitution(self.restitution)
            .density(self.density)
            .build();

        let ball_handle = world.add_body(ball_body, ball_name);
        world.add_collider(ball_collider, ball_handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simuforge_core::PhysicsConfig;

    #[test]
    fn test_bouncing_ball_setup() {
        let config = PhysicsConfig::default();
        let mut world = MetricWorld::new(&config);

        let scenario = BouncingBallScenario::default();
        scenario.setup(&mut world);

        assert_eq!(world.body_count(), 2);
    }
}
