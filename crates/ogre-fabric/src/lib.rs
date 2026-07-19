use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OgreFabricError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Key not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, OgreFabricError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeModification {
    pub file_path: String,
    pub diff: String,
    pub reasoning: String,
    pub test_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunStatus {
    Success,
    Failed(String),
    Running,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: String,
    pub task: String,
    pub decisions: Vec<String>,
    pub execution_time_ms: u64,
    pub token_cost: u32,
    pub status: RunStatus,
}

#[async_trait]
pub trait AgentPersistence: Send + Sync {
    async fn record_run(&self, run: &AgentRun) -> Result<String>;
    async fn record_modification(&self, mod_record: &CodeModification) -> Result<String>;
    async fn get_run_history(&self, limit: usize) -> Result<Vec<AgentRun>>;
    async fn query_changes(&self, path: Option<&str>) -> Result<Vec<CodeModification>>;
    async fn store_reasoning(&self, run_id: &str, reasoning: &str) -> Result<()>;
}

/// An in-memory/local persister that mocks storage in data-fabric.
pub struct MemoryAgentPersistence {
    runs: Arc<Mutex<HashMap<String, AgentRun>>>,
    modifications: Arc<Mutex<Vec<CodeModification>>>,
    reasoning: Arc<Mutex<HashMap<String, String>>>,
}

impl MemoryAgentPersistence {
    pub fn new() -> Self {
        Self {
            runs: Arc::new(Mutex::new(HashMap::new())),
            modifications: Arc::new(Mutex::new(Vec::new())),
            reasoning: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for MemoryAgentPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentPersistence for MemoryAgentPersistence {
    async fn record_run(&self, run: &AgentRun) -> Result<String> {
        let mut runs = self.runs.lock().unwrap();
        runs.insert(run.id.clone(), run.clone());
        Ok(run.id.clone())
    }

    async fn record_modification(&self, mod_record: &CodeModification) -> Result<String> {
        let mut modifications = self.modifications.lock().unwrap();
        modifications.push(mod_record.clone());
        Ok(mod_record.file_path.clone())
    }

    async fn get_run_history(&self, limit: usize) -> Result<Vec<AgentRun>> {
        let runs = self.runs.lock().unwrap();
        let list: Vec<AgentRun> = runs.values().take(limit).cloned().collect();
        Ok(list)
    }

    async fn query_changes(&self, path: Option<&str>) -> Result<Vec<CodeModification>> {
        let modifications = self.modifications.lock().unwrap();
        let filtered = modifications
            .iter()
            .filter(|m| path.is_none_or(|p| m.file_path.contains(p)))
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn store_reasoning(&self, run_id: &str, reasoning: &str) -> Result<()> {
        let mut r = self.reasoning.lock().unwrap();
        r.insert(run_id.to_string(), reasoning.to_string());
        Ok(())
    }
}
