//! SimuForge CLI - Physics experiment harness

use clap::{Parser, Subcommand};
use anyhow::Result;

mod runner;
mod commands;

use commands::{run, baseline, suite};

#[derive(Parser)]
#[command(name = "simuforge")]
#[command(author, version, about = "Physics simulation harness for structured experiments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single experiment
    Run {
        /// Path to experiment YAML file
        experiment: String,

        /// Output file for results (default: stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Baseline file to compare against
        #[arg(short, long)]
        baseline: Option<String>,

        /// Include per-frame metrics in output
        #[arg(long)]
        frames: bool,

        /// Pretty-print JSON output
        #[arg(long)]
        pretty: bool,
    },

    /// Generate a baseline from an experiment
    Baseline {
        /// Path to experiment YAML file
        experiment: String,

        /// Output file for baseline
        #[arg(short, long)]
        output: String,
    },

    /// Run a suite of experiments
    Suite {
        /// Directory containing experiment files
        directory: String,

        /// Output directory for results
        #[arg(short, long, default_value = "results")]
        output: String,

        /// Stop on first failure
        #[arg(long)]
        fail_fast: bool,
    },

    /// List available built-in scenarios
    Scenarios,

    /// Validate an experiment file
    Validate {
        /// Path to experiment YAML file
        experiment: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            experiment,
            output,
            baseline,
            frames,
            pretty,
        } => run::execute(&experiment, output.as_deref(), baseline.as_deref(), frames, pretty),

        Commands::Baseline { experiment, output } => baseline::execute(&experiment, &output),

        Commands::Suite {
            directory,
            output,
            fail_fast,
        } => suite::execute(&directory, &output, fail_fast),

        Commands::Scenarios => {
            println!("Available built-in scenarios:");
            println!("  box_stack      - Stack of boxes on ground plane");
            println!("  rolling_sphere - Sphere rolling on flat surface");
            println!("  bouncing_ball  - Ball dropped from height");
            println!("  friction_ramp  - Object sliding down inclined ramp");
            Ok(())
        }

        Commands::Validate { experiment } => {
            let spec = runner::load_experiment(&experiment)?;
            match spec.validate() {
                Ok(()) => {
                    println!("✓ Experiment specification is valid");
                    Ok(())
                }
                Err(errors) => {
                    eprintln!("✗ Validation errors:");
                    for err in errors {
                        eprintln!("  - {}", err);
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
