//! Simulation report types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{AggregateMetrics, MetricFrame, spec::CriteriaConfig};

/// Final simulation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub status: ReportStatus,
    pub experiment_name: String,
    pub total_steps: u64,
    pub total_time: f32,
    pub metrics: AggregateMetrics,
    pub criteria_results: HashMap<String, CriterionResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_comparison: Option<BaselineComparison>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SimulationReport {
    pub fn new(experiment_name: String) -> Self {
        Self {
            status: ReportStatus::Pending,
            experiment_name,
            total_steps: 0,
            total_time: 0.0,
            metrics: AggregateMetrics::default(),
            criteria_results: HashMap::new(),
            baseline_comparison: None,
            error: None,
        }
    }

    pub fn with_error(experiment_name: String, error: String) -> Self {
        Self {
            status: ReportStatus::Error,
            experiment_name,
            total_steps: 0,
            total_time: 0.0,
            metrics: AggregateMetrics::default(),
            criteria_results: HashMap::new(),
            baseline_comparison: None,
            error: Some(error),
        }
    }

    pub fn finalize(
        &mut self,
        frames: &[MetricFrame],
        criteria: &HashMap<String, CriteriaConfig>,
    ) {
        if let Some(last_frame) = frames.last() {
            self.total_steps = last_frame.step;
            self.total_time = last_frame.time;
        }

        self.metrics = AggregateMetrics::compute(frames);
        self.evaluate_criteria(criteria);
    }

    fn evaluate_criteria(&mut self, criteria: &HashMap<String, CriteriaConfig>) {
        let metric_values = self.get_metric_values();
        let mut all_passed = true;
        let mut results = Vec::new();

        for (name, config) in criteria {
            if let Some(&value) = metric_values.get(name.as_str()) {
                let passed = config.evaluate(value);
                if !passed {
                    all_passed = false;
                }
                results.push((
                    name.clone(),
                    CriterionResult {
                        value,
                        min: config.min,
                        max: config.max,
                        passed,
                    },
                ));
            }
        }

        for (name, result) in results {
            self.criteria_results.insert(name, result);
        }

        self.status = if all_passed {
            ReportStatus::Passed
        } else {
            ReportStatus::Failed
        };
    }

    fn get_metric_values(&self) -> HashMap<&str, f64> {
        let mut values = HashMap::new();
        values.insert("energy_drift_percent", self.metrics.energy_drift_percent);
        values.insert("max_penetration_ever", self.metrics.max_penetration_ever as f64);
        values.insert("total_constraint_violations", self.metrics.total_constraint_violations as f64);
        values.insert("average_contact_count", self.metrics.average_contact_count as f64);
        if let Some(step) = self.metrics.stabilization_step {
            values.insert("stabilization_step", step as f64);
        }
        values
    }

    pub fn compare_baseline(&mut self, baseline: &SimulationReport) {
        let mut metrics_improved = Vec::new();
        let mut metrics_regressed = Vec::new();

        // Compare energy drift (lower absolute value is better)
        if self.metrics.energy_drift_percent.abs() < baseline.metrics.energy_drift_percent.abs() {
            metrics_improved.push("energy_drift".to_string());
        } else if self.metrics.energy_drift_percent.abs() > baseline.metrics.energy_drift_percent.abs() {
            metrics_regressed.push("energy_drift".to_string());
        }

        // Compare max penetration (lower is better)
        if self.metrics.max_penetration_ever < baseline.metrics.max_penetration_ever {
            metrics_improved.push("max_penetration".to_string());
        } else if self.metrics.max_penetration_ever > baseline.metrics.max_penetration_ever {
            metrics_regressed.push("max_penetration".to_string());
        }

        // Compare constraint violations (lower is better)
        if self.metrics.total_constraint_violations < baseline.metrics.total_constraint_violations {
            metrics_improved.push("constraint_violations".to_string());
        } else if self.metrics.total_constraint_violations > baseline.metrics.total_constraint_violations {
            metrics_regressed.push("constraint_violations".to_string());
        }

        let recommendation = if metrics_regressed.is_empty() && !metrics_improved.is_empty() {
            ComparisonRecommendation::Accept
        } else if !metrics_regressed.is_empty() && metrics_improved.is_empty() {
            ComparisonRecommendation::Reject
        } else {
            ComparisonRecommendation::Review
        };

        self.baseline_comparison = Some(BaselineComparison {
            baseline_name: baseline.experiment_name.clone(),
            metrics_improved,
            metrics_regressed,
            recommendation,
        });
    }
}

/// Report status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportStatus {
    Pending,
    Passed,
    Failed,
    Error,
}

/// Result of evaluating a single criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    pub passed: bool,
}

/// Comparison against baseline results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub baseline_name: String,
    pub metrics_improved: Vec<String>,
    pub metrics_regressed: Vec<String>,
    pub recommendation: ComparisonRecommendation,
}

/// Recommendation based on baseline comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ComparisonRecommendation {
    Accept,
    Reject,
    Review,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_status() {
        let report = SimulationReport::new("test".to_string());
        assert_eq!(report.status, ReportStatus::Pending);
    }

    #[test]
    fn test_baseline_comparison() {
        let mut current = SimulationReport::new("current".to_string());
        current.metrics.energy_drift_percent = -1.0;
        current.metrics.max_penetration_ever = 0.001;

        let mut baseline = SimulationReport::new("baseline".to_string());
        baseline.metrics.energy_drift_percent = -3.0;
        baseline.metrics.max_penetration_ever = 0.005;

        current.compare_baseline(&baseline);

        let comparison = current.baseline_comparison.as_ref().unwrap();
        assert!(comparison.metrics_improved.contains(&"energy_drift".to_string()));
        assert!(comparison.metrics_improved.contains(&"max_penetration".to_string()));
        assert_eq!(comparison.recommendation, ComparisonRecommendation::Accept);
    }
}
