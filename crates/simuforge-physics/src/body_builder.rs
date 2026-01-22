//! Fluent API for building physics bodies

use rapier3d::prelude::*;
use nalgebra::UnitQuaternion;
use simuforge_core::{Vec3, spec::{BodyConfig, BodyType as SpecBodyType, ShapeConfig, MaterialConfig}};

/// Builder for creating physics bodies with colliders
pub struct BodyBuilder {
    name: String,
    position: Vector<f32>,
    rotation: UnitQuaternion<f32>,
    velocity: Vector<f32>,
    angular_velocity: Vector<f32>,
    body_type: RigidBodyType,
    shape: Option<SharedShape>,
    friction: f32,
    restitution: f32,
    density: f32,
}

impl BodyBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            position: Vector::zeros(),
            rotation: UnitQuaternion::identity(),
            velocity: Vector::zeros(),
            angular_velocity: Vector::zeros(),
            body_type: RigidBodyType::Dynamic,
            shape: None,
            friction: 0.5,
            restitution: 0.3,
            density: 1.0,
        }
    }

    pub fn from_config(config: &BodyConfig) -> Self {
        let mut builder = Self::new(&config.name)
            .position(config.position)
            .body_type(match config.body_type {
                SpecBodyType::Dynamic => RigidBodyType::Dynamic,
                SpecBodyType::Static => RigidBodyType::Fixed,
                SpecBodyType::Kinematic => RigidBodyType::KinematicPositionBased,
            })
            .material(&config.material);

        if let Some(rotation) = config.rotation {
            builder = builder.rotation_quat(rotation[0], rotation[1], rotation[2], rotation[3]);
        }

        if let Some(vel) = config.velocity {
            builder = builder.velocity(vel);
        }

        if let Some(angvel) = config.angular_velocity {
            builder = builder.angular_velocity(angvel);
        }

        builder = match &config.shape {
            ShapeConfig::Box { half_extents } => builder.box_shape(half_extents.x, half_extents.y, half_extents.z),
            ShapeConfig::Sphere { radius } => builder.sphere(*radius),
            ShapeConfig::Capsule { half_height, radius } => builder.capsule(*half_height, *radius),
            ShapeConfig::Cylinder { half_height, radius } => builder.cylinder(*half_height, *radius),
        };

        builder
    }

    pub fn position(mut self, pos: Vec3) -> Self {
        self.position = vector![pos.x, pos.y, pos.z];
        self
    }

    pub fn position_xyz(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = vector![x, y, z];
        self
    }

    pub fn rotation(mut self, axis: Vec3, angle: f32) -> Self {
        self.rotation = UnitQuaternion::from_axis_angle(
            &nalgebra::Unit::new_normalize(vector![axis.x, axis.y, axis.z]),
            angle,
        );
        self
    }

    pub fn rotation_quat(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.rotation = UnitQuaternion::from_quaternion(
            nalgebra::Quaternion::new(w, x, y, z)
        );
        self
    }

    pub fn velocity(mut self, vel: Vec3) -> Self {
        self.velocity = vector![vel.x, vel.y, vel.z];
        self
    }

    pub fn velocity_xyz(mut self, x: f32, y: f32, z: f32) -> Self {
        self.velocity = vector![x, y, z];
        self
    }

    pub fn angular_velocity(mut self, vel: Vec3) -> Self {
        self.angular_velocity = vector![vel.x, vel.y, vel.z];
        self
    }

    pub fn body_type(mut self, body_type: RigidBodyType) -> Self {
        self.body_type = body_type;
        self
    }

    pub fn dynamic(mut self) -> Self {
        self.body_type = RigidBodyType::Dynamic;
        self
    }

    pub fn fixed(mut self) -> Self {
        self.body_type = RigidBodyType::Fixed;
        self
    }

    pub fn kinematic(mut self) -> Self {
        self.body_type = RigidBodyType::KinematicPositionBased;
        self
    }

    pub fn box_shape(mut self, half_x: f32, half_y: f32, half_z: f32) -> Self {
        self.shape = Some(SharedShape::cuboid(half_x, half_y, half_z));
        self
    }

    pub fn sphere(mut self, radius: f32) -> Self {
        self.shape = Some(SharedShape::ball(radius));
        self
    }

    pub fn capsule(mut self, half_height: f32, radius: f32) -> Self {
        self.shape = Some(SharedShape::capsule_y(half_height, radius));
        self
    }

    pub fn cylinder(mut self, half_height: f32, radius: f32) -> Self {
        self.shape = Some(SharedShape::cylinder(half_height, radius));
        self
    }

    pub fn friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    pub fn restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    pub fn density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    pub fn material(mut self, material: &MaterialConfig) -> Self {
        self.friction = material.friction;
        self.restitution = material.restitution;
        self.density = material.density;
        self
    }

    /// Build and return the rigid body and collider
    pub fn build(self) -> (RigidBody, Collider, String) {
        let body = RigidBodyBuilder::new(self.body_type)
            .translation(self.position)
            .rotation(self.rotation.scaled_axis())
            .linvel(self.velocity)
            .angvel(self.angular_velocity)
            .build();

        let shape = self.shape.unwrap_or_else(|| SharedShape::cuboid(0.5, 0.5, 0.5));

        let collider = ColliderBuilder::new(shape)
            .friction(self.friction)
            .restitution(self.restitution)
            .density(self.density)
            .build();

        (body, collider, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_builder() {
        let (body, collider, name) = BodyBuilder::new("test")
            .position_xyz(0.0, 5.0, 0.0)
            .box_shape(0.5, 0.5, 0.5)
            .dynamic()
            .build();

        assert_eq!(name, "test");
        assert!(body.is_dynamic());
    }
}
