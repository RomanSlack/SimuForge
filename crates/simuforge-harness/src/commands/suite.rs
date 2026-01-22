//! Suite command implementation

use anyhow::Result;
use std::fs;
use std::path::Path;
use crate::runner::{load_experiment, run_experiment, save_report};
use simuforge_core::report::ReportStatus;

/// Execute the suite command
pub fn execute(directory: &str, output_dir: &str, fail_fast: bool) -> Result<()> {
    // Ensure output directory exists
    fs::create_dir_all(output_dir)?;

    // Find all YAML files in directory
    let experiments: Vec<_> = fs::read_dir(directory)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension()
                .map(|ext| ext == "yaml" || ext == "yml")
                .unwrap_or(false)
        })
        .collect();

    if experiments.is_empty() {
        eprintln!("No experiment files found in: {}", directory);
        return Ok(());
    }

    eprintln!("Found {} experiments", experiments.len());
    eprintln!();

    let mut passed = 0;
    let mut failed = 0;
    let mut errors = 0;

    for entry in &experiments {
        let path = entry.path();
        let name = path.file_stem().unwrap().to_string_lossy();

        eprint!("Running {}... ", name);

        match run_single_experiment(&path, output_dir) {
            Ok(status) => {
                match status {
                    ReportStatus::Passed => {
                        eprintln!("✓ PASSED");
                        passed += 1;
                    }
                    ReportStatus::Failed => {
                        eprintln!("✗ FAILED");
                        failed += 1;
                        if fail_fast {
                            eprintln!("Stopping due to --fail-fast");
                            break;
                        }
                    }
                    _ => {
                        eprintln!("? UNKNOWN");
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ ERROR: {}", e);
                errors += 1;
                if fail_fast {
                    eprintln!("Stopping due to --fail-fast");
                    break;
                }
            }
        }
    }

    eprintln!();
    eprintln!("=== Suite Summary ===");
    eprintln!("Passed:  {}", passed);
    eprintln!("Failed:  {}", failed);
    eprintln!("Errors:  {}", errors);
    eprintln!("Total:   {}", experiments.len());

    if failed > 0 || errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_single_experiment(path: &Path, output_dir: &str) -> Result<ReportStatus> {
    let spec = load_experiment(path.to_str().unwrap())?;
    spec.validate().map_err(|e| anyhow::anyhow!("Validation: {}", e.join(", ")))?;

    let (report, _frames) = run_experiment(&spec)?;

    // Save individual report
    let output_path = Path::new(output_dir)
        .join(format!("{}.json", spec.metadata.name));
    save_report(&report, output_path.to_str().unwrap(), true)?;

    Ok(report.status)
}
