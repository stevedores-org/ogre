# Issue #26: Define OGRE Integration Contracts

## Integration Contracts Defined ✅

### 1. oxidizedRAG ↔ OGRE Retrieval Interface

```rust
pub trait CodeRetriever: Send + Sync {
    async fn query_code(&self, query: &str, top_k: usize) -> Result<Vec<CodeContext>>;
    async fn get_code_by_location(&self, path: &str, identifier: &str) -> Result<CodeContext>;
    async fn find_callers(&self, path: &str, fn_name: &str) -> Result<Vec<CodeLocation>>;
    async fn analyze_change_impact(&self, path: &str, old: &str, new: &str) -> Result<ChangeImpact>;
}

pub struct CodeContext {
    pub path: String,
    pub language: Language,
    pub snippet: String,
    pub semantic_type: SemanticType,
    pub dependencies: Vec<Dependency>,
    pub relevance_score: f32,
}
```

### 2. OGRE ↔ oxidizedgraph Workflow Interface

```rust
pub trait AgentWorkflow: Send + Sync {
    async fn execute(&self, initial_state: AgentState, checkpoint_dir: Option<&str>) -> Result<AgentState>;
    async fn pause_for_approval(&self, decision: &str) -> Result<bool>;
    fn create_checkpoint(&self, state: &AgentState) -> Result<CheckpointId>;
    async fn resume_from_checkpoint(&self, checkpoint_id: &str) -> Result<AgentState>;
}

pub struct AgentState {
    pub messages: Vec<Message>,
    pub tool_calls: Vec<ToolCall>,
    pub context: HashMap<String, Value>,
    pub iteration: usize,
    pub is_complete: bool,
    pub user_approval_required: bool,
}
```

### 3. OGRE ↔ data-fabric Persistence Interface

```rust
pub trait AgentPersistence: Send + Sync {
    async fn record_run(&self, run: &AgentRun) -> Result<RunId>;
    async fn record_modification(&self, mod_record: &CodeModification) -> Result<ModificationId>;
    async fn get_run_history(&self, limit: usize) -> Result<Vec<AgentRun>>;
    async fn query_changes(&self, path: Option<&str>, from: DateTime, to: DateTime) -> Result<Vec<CodeModification>>;
    async fn store_reasoning(&self, run_id: &str, reasoning: &str) -> Result<()>;
}

pub struct AgentRun {
    pub id: String,
    pub task: String,
    pub initial_state: AgentState,
    pub final_state: AgentState,
    pub decisions: Vec<Decision>,
    pub execution_time_ms: u64,
    pub token_cost: u32,
    pub status: RunStatus,
}
```

### Integration Point Matrix

| Component | Interface | Direction | Purpose |
|-----------|-----------|-----------|---------|
| oxidizedRAG | CodeRetriever | ← OGRE | Retrieve code context |
| oxidizedgraph | AgentWorkflow | ← OGRE | Execute workflows |
| data-fabric | AgentPersistence | ← OGRE | Store runs & modifications |

### Technical Constraints

✅ All traits require tokio async (shared across projects)
✅ Error handling via thiserror + anyhow
✅ Serialization via Serde (matches data-fabric)
✅ Type safety via Rust generics

### Phase 2 Implementation Order

1. **Issue #27**: Implement code-specific retrieval layer (CodeRetriever)
2. **Issue #28**: Design safe action execution (AgentWorkflow extension)
3. **Issue #32**: Implement OGRE core (ties traits together)

---

**Assessment Document**: See [PHASE_1_ASSESSMENT.md](./PHASE_1_ASSESSMENT.md) for complete analysis.
