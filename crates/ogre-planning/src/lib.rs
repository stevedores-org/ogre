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
        let (steps, complexity, risk_level) =
            if task_description.contains("refactor") || task_description.contains("rewrite") {
                (
                    vec![
                        "Scan codebase for usages".to_string(),
                        "Extract functions to common helper module".to_string(),
                        "Update all caller files".to_string(),
                        "Run cargo test to verify refactoring".to_string(),
                    ],
                    7,
                    "medium".to_string(),
                )
            } else if task_description.contains("credential") || task_description.contains("secret") {
                (
                    vec![
                        "Identify secret exposure".to_string(),
                        "Remove secret from source code".to_string(),
                        "Add environment variable config".to_string(),
                    ],
                    3,
                    "high".to_string(),
                )
            } else {
                (
                    vec![
                        "Analyze request requirements".to_string(),
                        "Apply simple file modifications".to_string(),
                        "Validate code via cargo check".to_string(),
                    ],
                    2,
                    "low".to_string(),
                )
            };

        Ok(Plan {
            steps,
            complexity,
            risk_level,
        })
    }
}
