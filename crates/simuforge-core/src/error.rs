//! Error types for SimuForge

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimuForgeError {
    #[error("Invalid experiment specification: {0}")]
    InvalidSpec(String),

    #[error("Scenario not found: {0}")]
    ScenarioNotFound(String),

    #[error("Physics error: {0}")]
    Physics(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Baseline mismatch: {0}")]
    BaselineMismatch(String),
}

pub type Result<T> = std::result::Result<T, SimuForgeError>;
