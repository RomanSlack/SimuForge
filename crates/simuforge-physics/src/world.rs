//! MetricWorld - Rapier wrapper with metric collection

use rapier3d::prelude::*;
use simuforge_core::{
    Vec3, Transform, MetricFrame, EnergyMetrics, MomentumMetrics, ContactMetrics,
    metrics::BodyState, PhysicsConfig, ExperimentSpec,
};
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// Physics world wrapper that collects metrics each step
pub struct MetricWorld {
    // Rapier components
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub gravity: Vector<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,

    // Simulation state
    pub current_step: u64,
    pub current_time: f32,
    timestep: f32,

    // Body tracking
    body_names: HashMap<RigidBodyHandle, String>,
    body_ids: HashMap<RigidBodyHandle, u64>,
    next_body_id: u64,

    // Metric collection
    frames: Vec<MetricFrame>,
    collect_body_states: bool,
}

impl MetricWorld {
    /// Create a new physics world with the given configuration
    pub fn new(config: &PhysicsConfig) -> Self {
        let gravity = vector![config.gravity.x, config.gravity.y, config.gravity.z];

        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = config.timestep;
        integration_parameters.num_solver_iterations = NonZeroUsize::new(config.solver_iterations as usize)
            .unwrap_or(NonZeroUsize::new(8).unwrap());

        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity,
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            current_step: 0,
            current_time: 0.0,
            timestep: config.timestep,
            body_names: HashMap::new(),
            body_ids: HashMap::new(),
            next_body_id: 0,
            frames: Vec::new(),
            collect_body_states: true,
        }
    }

    /// Create world from experiment specification
    pub fn from_spec(spec: &ExperimentSpec) -> Self {
        Self::new(&spec.spec.physics)
    }

    /// Set whether to collect full body states each frame
    pub fn set_collect_body_states(&mut self, collect: bool) {
        self.collect_body_states = collect;
    }

    /// Add a rigid body to the world
    pub fn add_body(&mut self, body: RigidBody, name: String) -> RigidBodyHandle {
        let handle = self.rigid_body_set.insert(body);
        let id = self.next_body_id;
        self.next_body_id += 1;
        self.body_names.insert(handle, name);
        self.body_ids.insert(handle, id);
        handle
    }

    /// Add a collider to a body
    pub fn add_collider(&mut self, collider: Collider, parent: RigidBodyHandle) -> ColliderHandle {
        self.collider_set.insert_with_parent(collider, parent, &mut self.rigid_body_set)
    }

    /// Step the simulation forward
    pub fn step(&mut self) {
        // Collect pre-step metrics
        let frame = self.collect_metrics();
        self.frames.push(frame);

        // Step physics
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );

        self.current_step += 1;
        self.current_time += self.timestep;
    }

    /// Run simulation for specified number of steps
    pub fn run(&mut self, steps: u64) {
        for _ in 0..steps {
            self.step();
        }
        // Collect final frame
        let frame = self.collect_metrics();
        self.frames.push(frame);
    }

    /// Get collected metric frames
    pub fn frames(&self) -> &[MetricFrame] {
        &self.frames
    }

    /// Get the current metric frame
    pub fn current_frame(&self) -> MetricFrame {
        self.collect_metrics()
    }

    /// Clear collected frames
    pub fn clear_frames(&mut self) {
        self.frames.clear();
    }

    /// Collect metrics for the current state
    fn collect_metrics(&self) -> MetricFrame {
        let mut frame = MetricFrame::new(self.current_step, self.current_time);

        // Energy metrics
        frame.energy = self.compute_energy();

        // Momentum metrics
        frame.momentum = self.compute_momentum();

        // Contact metrics
        frame.contacts = self.compute_contacts();

        // Body states
        if self.collect_body_states {
            frame.bodies = self.collect_body_states();
        }

        frame
    }

    /// Compute total kinetic and potential energy
    fn compute_energy(&self) -> EnergyMetrics {
        let mut kinetic = 0.0f32;
        let mut potential = 0.0f32;

        for (handle, body) in self.rigid_body_set.iter() {
            if body.is_dynamic() {
                let mass = body.mass();
                let vel = body.linvel();
                let angvel = body.angvel();

                // Kinetic energy: 0.5 * m * v^2 + 0.5 * I * w^2
                kinetic += 0.5 * mass * vel.norm_squared();

                // Rotational kinetic energy (simplified, assuming uniform sphere inertia)
                let inertia = mass * 0.4; // Approximate
                kinetic += 0.5 * inertia * angvel.norm_squared();

                // Potential energy: m * g * h (relative to y=0)
                let height = body.translation().y;
                let g = self.gravity.y.abs();
                potential += mass * g * height;
            }
        }

        EnergyMetrics::new(kinetic, potential)
    }

    /// Compute total linear and angular momentum
    fn compute_momentum(&self) -> MomentumMetrics {
        let mut linear = nalgebra::Vector3::zeros();
        let mut angular = nalgebra::Vector3::zeros();

        for (_handle, body) in self.rigid_body_set.iter() {
            if body.is_dynamic() {
                let mass = body.mass();

                // Linear momentum: m * v
                linear += mass * body.linvel();

                // Angular momentum (simplified): I * w
                let inertia = mass * 0.4;
                angular += inertia * body.angvel();
            }
        }

        MomentumMetrics::new(
            Vec3::from_nalgebra(&linear),
            Vec3::from_nalgebra(&angular),
        )
    }

    /// Compute contact metrics
    fn compute_contacts(&self) -> ContactMetrics {
        let mut metrics = ContactMetrics::default();

        for pair in self.narrow_phase.contact_pairs() {
            if pair.has_any_active_contact {
                metrics.contact_count += 1;

                for manifold in pair.manifolds.iter() {
                    for point in manifold.points.iter() {
                        let penetration = -point.dist;
                        if penetration > 0.0 {
                            metrics.max_penetration = metrics.max_penetration.max(penetration);
                            metrics.total_penetration += penetration;
                        }
                    }
                }
            }
        }

        metrics
    }

    /// Collect state of all bodies
    fn collect_body_states(&self) -> Vec<BodyState> {
        self.rigid_body_set
            .iter()
            .filter_map(|(handle, body)| {
                let id = self.body_ids.get(&handle)?;
                let name = self.body_names.get(&handle)?;

                Some(BodyState {
                    id: *id,
                    name: name.clone(),
                    transform: Transform::from_isometry(body.position()),
                    velocity: Vec3::from_nalgebra(body.linvel()),
                    angular_velocity: Vec3::from_nalgebra(body.angvel()),
                    sleeping: body.is_sleeping(),
                })
            })
            .collect()
    }

    /// Get body by name
    pub fn get_body_by_name(&self, name: &str) -> Option<&RigidBody> {
        self.body_names
            .iter()
            .find(|(_, n)| *n == name)
            .and_then(|(handle, _)| self.rigid_body_set.get(*handle))
    }

    /// Get mutable body by name
    pub fn get_body_by_name_mut(&mut self, name: &str) -> Option<&mut RigidBody> {
        let handle = self.body_names
            .iter()
            .find(|(_, n)| *n == name)
            .map(|(handle, _)| *handle)?;
        self.rigid_body_set.get_mut(handle)
    }

    /// Get number of bodies
    pub fn body_count(&self) -> usize {
        self.rigid_body_set.len()
    }

    /// Get current simulation time
    pub fn time(&self) -> f32 {
        self.current_time
    }

    /// Get current step
    pub fn step_count(&self) -> u64 {
        self.current_step
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simuforge_core::PhysicsConfig;

    #[test]
    fn test_world_creation() {
        let config = PhysicsConfig::default();
        let world = MetricWorld::new(&config);
        assert_eq!(world.body_count(), 0);
    }

    #[test]
    fn test_add_body() {
        let config = PhysicsConfig::default();
        let mut world = MetricWorld::new(&config);

        let body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 5.0, 0.0])
            .build();
        world.add_body(body, "test_body".to_string());

        assert_eq!(world.body_count(), 1);
    }

    #[test]
    fn test_step_simulation() {
        let config = PhysicsConfig::default();
        let mut world = MetricWorld::new(&config);

        let body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 5.0, 0.0])
            .build();
        let handle = world.add_body(body, "falling_box".to_string());

        let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5).build();
        world.add_collider(collider, handle);

        world.step();
        assert_eq!(world.step_count(), 1);
        assert_eq!(world.frames().len(), 1);
    }
}
