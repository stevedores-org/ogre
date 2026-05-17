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
        match (&self.state, &new_state) {
            (AgentState::Init, AgentState::Plan) => {}
            (AgentState::Plan, AgentState::Execute) => {}
            (AgentState::Execute, AgentState::Validate) => {}
            (AgentState::Validate, AgentState::Completed) => {}
            (AgentState::Validate, AgentState::Plan) => {} // Retry planning
            (_, AgentState::Failed(_)) => {} // Any state can fail
            (current, next) => {
                return Err(OgreCoreError::InvalidStateTransition {
                    from: current.clone(),
                    to: next.clone(),
                });
            }
        }
        
        self.state = new_state;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let mut ctx = AgentContext::new("/path", "Test task");
        assert_eq!(ctx.state, AgentState::Init);

        assert!(ctx.transition(AgentState::Plan).is_ok());
        assert_eq!(ctx.state, AgentState::Plan);

        assert!(ctx.transition(AgentState::Execute).is_ok());
        assert_eq!(ctx.state, AgentState::Execute);

        assert!(ctx.transition(AgentState::Validate).is_ok());
        assert_eq!(ctx.state, AgentState::Validate);

        assert!(ctx.transition(AgentState::Completed).is_ok());
        assert_eq!(ctx.state, AgentState::Completed);
    }

    #[test]
    fn test_retry_planning() {
        let mut ctx = AgentContext::new("/path", "Test task");
        ctx.transition(AgentState::Plan).unwrap();
        ctx.transition(AgentState::Execute).unwrap();
        ctx.transition(AgentState::Validate).unwrap();
        
        // Validation failed, need to replan
        assert!(ctx.transition(AgentState::Plan).is_ok());
        assert_eq!(ctx.state, AgentState::Plan);
    }

    #[test]
    fn test_invalid_transitions() {
        let mut ctx = AgentContext::new("/path", "Test task");
        
        // Cannot go from Init to Execute directly
        let err = ctx.transition(AgentState::Execute).unwrap_err();
        match err {
            OgreCoreError::InvalidStateTransition { from, to } => {
                assert_eq!(from, AgentState::Init);
                assert_eq!(to, AgentState::Execute);
            }
            _ => panic!("Expected InvalidStateTransition error"),
        }
    }

    #[test]
    fn test_failure_transition() {
        let mut ctx = AgentContext::new("/path", "Test task");
        assert!(ctx.transition(AgentState::Failed("Some error".into())).is_ok());
        assert_eq!(ctx.state, AgentState::Failed("Some error".into()));
    }
}
