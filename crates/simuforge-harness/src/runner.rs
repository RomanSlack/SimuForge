//! Experiment runner

use anyhow::{Context, Result};
use simuforge_core::{
    ExperimentSpec, SimulationReport, MetricFrame,
    spec::DurationConfig,
};
use simuforge_physics::{MetricWorld, create_scenario};
use std::fs;
use std::path::Path;

/// Load experiment specification from YAML file
pub fn load_experiment(path: &str) -> Result<ExperimentSpec> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read experiment file: {}", path))?;

    let spec: ExperimentSpec = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse experiment YAML: {}", path))?;

    Ok(spec)
}

/// Load baseline report from JSON file
pub fn load_baseline(path: &str) -> Result<SimulationReport> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read baseline file: {}", path))?;

    let report: SimulationReport = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse baseline JSON: {}", path))?;

    Ok(report)
}

/// Save report to JSON file
pub fn save_report(report: &SimulationReport, path: &str, pretty: bool) -> Result<()> {
    let content = if pretty {
        serde_json::to_string_pretty(report)?
    } else {
        serde_json::to_string(report)?
    };

    fs::write(path, content)
        .with_context(|| format!("Failed to write report: {}", path))?;

    Ok(())
}

/// Run an experiment and return the report
pub fn run_experiment(spec: &ExperimentSpec) -> Result<(SimulationReport, Vec<MetricFrame>)> {
    // Create physics world
    let mut world = MetricWorld::from_spec(spec);

    // Set up scenario
    let scenario = create_scenario(&spec.spec.scenario);
    scenario.setup(&mut world);

    // Determine step count
    let steps = match &spec.spec.duration {
        DurationConfig::Fixed { steps } => *steps,
        DurationConfig::Time { seconds } => {
            (*seconds / spec.spec.physics.timestep) as u64
        }
        DurationConfig::UntilStable { max_steps, .. } => *max_steps,
    };

    // Run simulation
    world.run(steps);

    // Build report
    let frames = world.frames().to_vec();
    let mut report = SimulationReport::new(spec.metadata.name.clone());
    report.finalize(&frames, &spec.spec.criteria);

    Ok((report, frames))
}

/// Result of running an experiment
pub struct ExperimentResult {
    pub report: SimulationReport,
    pub frames: Vec<MetricFrame>,
    pub experiment_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_minimal_experiment() {
        let yaml = r#"
apiVersion: simuforge/v1
kind: Experiment
metadata:
  name: test
spec:
  physics:
    timestep: 0.016666667
  duration:
    type: fixed
    steps: 10
  scenario:
    type: builtin
    name: box_stack
    params:
      count: 2
"#;
        let spec: ExperimentSpec = serde_yaml::from_str(yaml).unwrap();
        let (report, frames) = run_experiment(&spec).unwrap();

        assert!(frames.len() > 0);
        assert_eq!(report.experiment_name, "test");
    }
}
