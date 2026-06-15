use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTrace {
    pub timestamp: DateTime<Utc>,
    pub node_id: String,
    pub action: String,
    pub input_state_hash: String,
    pub output_state_hash: String,
    pub token_cost: u32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowMetrics {
    pub total_steps: usize,
    pub total_tokens: u32,
    pub total_cost_usd: f32,
    pub total_duration_ms: u64,
}

pub struct AgentOtelTracer {
    traces: Arc<Mutex<Vec<DecisionTrace>>>,
    metrics: Arc<Mutex<WorkflowMetrics>>,
}

impl AgentOtelTracer {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(WorkflowMetrics::default())),
        }
    }

    pub fn record_decision(&self, node_id: &str, action: &str, reasoning: &str, token_cost: u32) {
        let mut traces = self.traces.lock().unwrap();
        traces.push(DecisionTrace {
            timestamp: Utc::now(),
            node_id: node_id.to_string(),
            action: action.to_string(),
            input_state_hash: "".to_string(), // can be computed in real impl
            output_state_hash: "".to_string(),
            token_cost,
            reasoning: reasoning.to_string(),
        });

        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_steps += 1;
        metrics.total_tokens += token_cost;
        metrics.total_cost_usd += (token_cost as f32) * 0.000002; // simulated cost
    }

    pub fn get_traces(&self) -> Vec<DecisionTrace> {
        self.traces.lock().unwrap().clone()
    }

    pub fn get_metrics(&self) -> WorkflowMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

impl Default for AgentOtelTracer {
    fn default() -> Self {
        Self::new()
    }
}
