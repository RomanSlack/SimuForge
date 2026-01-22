//! Baseline command implementation

use anyhow::Result;
use crate::runner::{load_experiment, run_experiment, save_report};

/// Execute the baseline command
pub fn execute(experiment_path: &str, output_path: &str) -> Result<()> {
    // Load experiment
    let spec = load_experiment(experiment_path)?;

    // Validate
    if let Err(errors) = spec.validate() {
        eprintln!("Validation errors:");
        for err in errors {
            eprintln!("  - {}", err);
        }
        anyhow::bail!("Invalid experiment specification");
    }

    eprintln!("Running experiment to generate baseline...");

    // Run experiment
    let (report, _frames) = run_experiment(&spec)?;

    // Save baseline
    save_report(&report, output_path, true)?;

    eprintln!("Baseline saved to: {}", output_path);
    eprintln!();
    eprintln!("Baseline metrics:");
    eprintln!("  Energy drift: {:.2}%", report.metrics.energy_drift_percent);
    eprintln!("  Max penetration: {:.6}", report.metrics.max_penetration_ever);
    eprintln!("  Constraint violations: {}", report.metrics.total_constraint_violations);

    Ok(())
}
