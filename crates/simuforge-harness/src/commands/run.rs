//! Run command implementation

use anyhow::Result;
use crate::runner::{load_experiment, load_baseline, run_experiment, save_report};
use serde::Serialize;
use simuforge_core::{SimulationReport, MetricFrame};

/// Extended report including optional frame data
#[derive(Serialize)]
struct ExtendedReport {
    #[serde(flatten)]
    report: SimulationReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    frames: Option<Vec<MetricFrame>>,
}

/// Execute the run command
pub fn execute(
    experiment_path: &str,
    output_path: Option<&str>,
    baseline_path: Option<&str>,
    include_frames: bool,
    pretty: bool,
) -> Result<()> {
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

    // Run experiment
    let (mut report, frames) = run_experiment(&spec)?;

    // Compare to baseline if provided
    if let Some(baseline_path) = baseline_path {
        let baseline = load_baseline(baseline_path)?;
        report.compare_baseline(&baseline);
    }

    // Build output
    let extended = ExtendedReport {
        report: report.clone(),
        frames: if include_frames { Some(frames) } else { None },
    };

    let output = if pretty {
        serde_json::to_string_pretty(&extended)?
    } else {
        serde_json::to_string(&extended)?
    };

    // Output results
    if let Some(path) = output_path {
        std::fs::write(path, &output)?;
        eprintln!("Results written to: {}", path);
    } else {
        println!("{}", output);
    }

    // Print summary to stderr
    eprintln!();
    eprintln!("=== Experiment Summary ===");
    eprintln!("Name: {}", report.experiment_name);
    eprintln!("Status: {:?}", report.status);
    eprintln!("Steps: {}", report.total_steps);
    eprintln!("Time: {:.3}s", report.total_time);
    eprintln!();
    eprintln!("Metrics:");
    eprintln!("  Energy drift: {:.2}%", report.metrics.energy_drift_percent);
    eprintln!("  Max penetration: {:.6}", report.metrics.max_penetration_ever);
    eprintln!("  Constraint violations: {}", report.metrics.total_constraint_violations);

    if !report.criteria_results.is_empty() {
        eprintln!();
        eprintln!("Criteria:");
        for (name, result) in &report.criteria_results {
            let status = if result.passed { "✓" } else { "✗" };
            eprintln!("  {} {}: {:.4}", status, name, result.value);
        }
    }

    if let Some(comparison) = &report.baseline_comparison {
        eprintln!();
        eprintln!("Baseline Comparison:");
        eprintln!("  Recommendation: {:?}", comparison.recommendation);
        if !comparison.metrics_improved.is_empty() {
            eprintln!("  Improved: {}", comparison.metrics_improved.join(", "));
        }
        if !comparison.metrics_regressed.is_empty() {
            eprintln!("  Regressed: {}", comparison.metrics_regressed.join(", "));
        }
    }

    Ok(())
}
