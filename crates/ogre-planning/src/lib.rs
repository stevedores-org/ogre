use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OgrePlanningError {
    #[error("Planning failed: {reason}")]
    PlanningFailed { reason: String },
}

pub type Result<T> = std::result::Result<T, OgrePlanningError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Plan {
    pub steps: Vec<String>,
    pub complexity: u32,
    pub risk_level: String,
}

pub struct TaskDecomposer {
    pub model_name: String,
}

impl TaskDecomposer {
    pub fn new(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
        }
    }

    pub fn decompose_task(&self, task_description: &str) -> Result<Plan> {
        let mut steps = Vec::new();
        let mut complexity = 1;
        let mut risk_level = "low".to_string();

        if task_description.contains("refactor") || task_description.contains("rewrite") {
            steps.push("Scan codebase for usages".to_string());
            steps.push("Extract functions to common helper module".to_string());
            steps.push("Update all caller files".to_string());
            steps.push("Run cargo test to verify refactoring".to_string());
            complexity = 7;
            risk_level = "medium".to_string();
        } else if task_description.contains("credential") || task_description.contains("secret") {
            steps.push("Identify secret exposure".to_string());
            steps.push("Remove secret from source code".to_string());
            steps.push("Add environment variable config".to_string());
            complexity = 3;
            risk_level = "high".to_string();
        } else {
            steps.push("Analyze request requirements".to_string());
            steps.push("Apply simple file modifications".to_string());
            steps.push("Validate code via cargo check".to_string());
            complexity = 2;
            risk_level = "low".to_string();
        }

        Ok(Plan {
            steps,
            complexity,
            risk_level,
        })
    }
}
