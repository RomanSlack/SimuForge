//! Metric types for simulation analysis

use serde::{Deserialize, Serialize};
use crate::{Vec3, Transform};

/// Per-frame metrics collected during simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricFrame {
    pub step: u64,
    pub time: f32,
    pub energy: EnergyMetrics,
    pub momentum: MomentumMetrics,
    pub contacts: ContactMetrics,
    #[serde(default)]
    pub bodies: Vec<BodyState>,
}

impl MetricFrame {
    pub fn new(step: u64, time: f32) -> Self {
        Self {
            step,
            time,
            energy: EnergyMetrics::default(),
            momentum: MomentumMetrics::default(),
            contacts: ContactMetrics::default(),
            bodies: Vec::new(),
        }
    }
}

/// Energy metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnergyMetrics {
    pub kinetic: f32,
    pub potential: f32,
    pub total: f32,
}

impl EnergyMetrics {
    pub fn new(kinetic: f32, potential: f32) -> Self {
        Self {
            kinetic,
            potential,
            total: kinetic + potential,
        }
    }
}

/// Momentum metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MomentumMetrics {
    pub linear: Vec3,
    pub angular: Vec3,
    pub linear_magnitude: f32,
    pub angular_magnitude: f32,
}

impl MomentumMetrics {
    pub fn new(linear: Vec3, angular: Vec3) -> Self {
        Self {
            linear,
            angular,
            linear_magnitude: linear.magnitude(),
            angular_magnitude: angular.magnitude(),
        }
    }
}

/// Contact/collision metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContactMetrics {
    pub contact_count: u32,
    pub max_penetration: f32,
    pub total_penetration: f32,
    pub constraint_violations: u32,
}

/// State of a single body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyState {
    pub id: u64,
    pub name: String,
    pub transform: Transform,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub sleeping: bool,
}

/// Aggregated metrics computed at the end of simulation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregateMetrics {
    pub energy_drift_percent: f64,
    pub initial_energy: f32,
    pub final_energy: f32,
    pub max_penetration_ever: f32,
    pub total_constraint_violations: u64,
    pub stabilization_step: Option<u64>,
    pub stability_time: Option<f32>,
    pub average_contact_count: f32,
    pub frame_count: u64,
}

impl AggregateMetrics {
    pub fn compute(frames: &[MetricFrame]) -> Self {
        if frames.is_empty() {
            return Self::default();
        }

        let initial_energy = frames.first().map(|f| f.energy.total).unwrap_or(0.0);
        let final_energy = frames.last().map(|f| f.energy.total).unwrap_or(0.0);

        let energy_drift_percent = if initial_energy.abs() > 1e-6 {
            ((final_energy - initial_energy) / initial_energy * 100.0) as f64
        } else {
            0.0
        };

        let max_penetration_ever = frames
            .iter()
            .map(|f| f.contacts.max_penetration)
            .fold(0.0f32, f32::max);

        let total_constraint_violations = frames
            .iter()
            .map(|f| f.contacts.constraint_violations as u64)
            .sum();

        let total_contacts: u64 = frames.iter().map(|f| f.contacts.contact_count as u64).sum();
        let average_contact_count = total_contacts as f32 / frames.len() as f32;

        // Find stabilization point (when all bodies are sleeping or velocity below threshold)
        let stabilization_step = frames.iter().position(|f| {
            f.bodies.iter().all(|b| b.sleeping || b.velocity.magnitude() < 0.01)
        }).map(|i| frames[i].step);

        let stability_time = stabilization_step
            .and_then(|step| frames.iter().find(|f| f.step == step))
            .map(|f| f.time);

        Self {
            energy_drift_percent,
            initial_energy,
            final_energy,
            max_penetration_ever,
            total_constraint_violations,
            stabilization_step,
            stability_time,
            average_contact_count,
            frame_count: frames.len() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_metrics() {
        let energy = EnergyMetrics::new(100.0, 50.0);
        assert_eq!(energy.total, 150.0);
    }

    #[test]
    fn test_aggregate_metrics() {
        let frames = vec![
            MetricFrame {
                step: 0,
                time: 0.0,
                energy: EnergyMetrics::new(100.0, 0.0),
                momentum: MomentumMetrics::default(),
                contacts: ContactMetrics::default(),
                bodies: vec![],
            },
            MetricFrame {
                step: 1,
                time: 0.016,
                energy: EnergyMetrics::new(98.0, 0.0),
                momentum: MomentumMetrics::default(),
                contacts: ContactMetrics { max_penetration: 0.001, ..Default::default() },
                bodies: vec![],
            },
        ];

        let agg = AggregateMetrics::compute(&frames);
        assert!((agg.energy_drift_percent - (-2.0)).abs() < 0.1);
        assert_eq!(agg.max_penetration_ever, 0.001);
    }
}
