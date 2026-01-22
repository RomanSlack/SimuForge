//! Metric collection implementations

use simuforge_core::{MetricFrame, AggregateMetrics};

/// Analyze metric frames for stability detection
pub fn detect_stability(frames: &[MetricFrame], velocity_threshold: f32) -> Option<u64> {
    frames.iter().position(|f| {
        f.bodies.iter().all(|b| {
            b.sleeping || (b.velocity.magnitude() < velocity_threshold && b.angular_velocity.magnitude() < velocity_threshold)
        })
    }).map(|i| frames[i].step)
}

/// Calculate energy conservation ratio
pub fn energy_conservation_ratio(frames: &[MetricFrame]) -> f64 {
    if frames.len() < 2 {
        return 1.0;
    }

    let initial = frames.first().map(|f| f.energy.total).unwrap_or(0.0);
    let final_energy = frames.last().map(|f| f.energy.total).unwrap_or(0.0);

    if initial.abs() < 1e-6 {
        return 1.0;
    }

    (final_energy / initial) as f64
}

/// Find the frame with maximum penetration
pub fn max_penetration_frame(frames: &[MetricFrame]) -> Option<&MetricFrame> {
    frames.iter().max_by(|a, b| {
        a.contacts.max_penetration.partial_cmp(&b.contacts.max_penetration).unwrap()
    })
}

/// Compute running average of a metric
pub fn running_average<F>(frames: &[MetricFrame], window: usize, extractor: F) -> Vec<f32>
where
    F: Fn(&MetricFrame) -> f32,
{
    if frames.len() < window {
        return frames.iter().map(&extractor).collect();
    }

    frames
        .windows(window)
        .map(|w| w.iter().map(&extractor).sum::<f32>() / window as f32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use simuforge_core::EnergyMetrics;

    #[test]
    fn test_energy_conservation() {
        let frames = vec![
            MetricFrame {
                step: 0,
                time: 0.0,
                energy: EnergyMetrics::new(100.0, 0.0),
                momentum: Default::default(),
                contacts: Default::default(),
                bodies: vec![],
            },
            MetricFrame {
                step: 1,
                time: 0.016,
                energy: EnergyMetrics::new(95.0, 0.0),
                momentum: Default::default(),
                contacts: Default::default(),
                bodies: vec![],
            },
        ];

        let ratio = energy_conservation_ratio(&frames);
        assert!((ratio - 0.95).abs() < 0.01);
    }
}
