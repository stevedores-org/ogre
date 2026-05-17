use crate::error::{OgreCoreError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentState {
    Init,
    Plan,
    Execute,
    Validate,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub agent_id: Uuid,
    pub state: AgentState,
    pub codebase_path: String,
    pub task_description: String,
}

impl AgentContext {
    pub fn new(codebase_path: &str, task_description: &str) -> Self {
        Self {
            agent_id: Uuid::new_v4(),
            state: AgentState::Init,
            codebase_path: codebase_path.to_string(),
            task_description: task_description.to_string(),
        }
    }

    pub fn transition(&mut self, new_state: AgentState) -> Result<()> {
        // Basic state machine validation
        match (&self.state, &new_state) {
            (AgentState::Init, AgentState::Plan) => {}
            (AgentState::Plan, AgentState::Execute) => {}
            (AgentState::Execute, AgentState::Validate) => {}
            (AgentState::Validate, AgentState::Completed) => {}
            (AgentState::Validate, AgentState::Plan) => {} // Retry planning on validation failure
            (_, AgentState::Failed(_)) => {}
            (current, next) => {
                return Err(OgreCoreError::LifecycleError(format!(
                    "Invalid state transition from {:?} to {:?}",
                    current, next
                )));
            }
        }
        
        self.state = new_state;
        Ok(())
    }
}
