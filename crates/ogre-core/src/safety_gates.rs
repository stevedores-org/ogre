use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;

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
    fn evaluate_plan(&self, plan: &Plan) -> impl Future<Output = Result<ApprovalStatus>> + Send;
}

pub struct DefaultSafetyGate {
    pub max_auto_complexity: u32,
}

impl SafetyGate for DefaultSafetyGate {
    async fn evaluate_plan(&self, plan: &Plan) -> Result<ApprovalStatus> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_safety_gate_approval() {
        let gate = DefaultSafetyGate { max_auto_complexity: 5 };
        
        // Low complexity, low risk -> Approved
        let plan_approved = Plan {
            steps: vec!["Format code".into()],
            complexity: 2,
            risk_level: "low".into(),
        };
        let status = gate.evaluate_plan(&plan_approved).await.unwrap();
        assert!(matches!(status, ApprovalStatus::Approved));

        // High complexity, low risk -> Pending
        let plan_pending_complexity = Plan {
            steps: vec!["Major refactor".into()],
            complexity: 8,
            risk_level: "low".into(),
        };
        let status = gate.evaluate_plan(&plan_pending_complexity).await.unwrap();
        assert!(matches!(status, ApprovalStatus::Pending));

        // Low complexity, high risk -> Pending
        let plan_pending_risk = Plan {
            steps: vec!["Update credentials".into()],
            complexity: 2,
            risk_level: "high".into(),
        };
        let status = gate.evaluate_plan(&plan_pending_risk).await.unwrap();
        assert!(matches!(status, ApprovalStatus::Pending));
    }
}
