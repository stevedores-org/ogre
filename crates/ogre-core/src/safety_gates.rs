use crate::error::{OgreCoreError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<String>,
    pub complexity: u32,
    pub risk_level: String,
}

pub trait SafetyGate {
    /// Validates if a plan meets automatic execution criteria or requires human approval.
    fn evaluate_plan(&self, plan: &Plan) -> Result<ApprovalStatus>;
}

pub struct DefaultSafetyGate {
    pub max_auto_complexity: u32,
}

impl SafetyGate for DefaultSafetyGate {
    fn evaluate_plan(&self, plan: &Plan) -> Result<ApprovalStatus> {
        if plan.risk_level == "high" {
            return Ok(ApprovalStatus::Pending);
        }
        
        if plan.complexity <= self.max_auto_complexity {
            Ok(ApprovalStatus::Approved)
        } else {
            Ok(ApprovalStatus::Pending)
        }
    }
}
