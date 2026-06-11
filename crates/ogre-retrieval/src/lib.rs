use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OgreRetrievalError {
    #[error("Retrieval failed: {0}")]
    RetrievalFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, OgreRetrievalError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    Go,
    TypeScript,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticType {
    Function,
    Struct,
    Module,
    Class,
    Block,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeContext {
    pub path: String,
    pub language: Language,
    pub snippet: String,
    pub semantic_type: SemanticType,
    pub dependencies: Vec<Dependency>,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeLocation {
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeImpact {
    pub file_path: String,
    pub affected_dependents: Vec<String>,
    pub breaking_risk: String, // "low", "medium", "high"
}

#[async_trait]
pub trait CodeRetriever: Send + Sync {
    async fn query_code(&self, query: &str, top_k: usize) -> Result<Vec<CodeContext>>;
    async fn get_code_by_location(&self, path: &str, identifier: &str) -> Result<CodeContext>;
    async fn find_callers(&self, path: &str, fn_name: &str) -> Result<Vec<CodeLocation>>;
    async fn analyze_change_impact(&self, path: &str, old: &str, new: &str) -> Result<ChangeImpact>;
}

/// A default / mock retriever that parses files locally or falls back to basic search.
pub struct DefaultCodeRetriever {
    pub root_dir: String,
}

impl DefaultCodeRetriever {
    pub fn new(root_dir: &str) -> Self {
        Self {
            root_dir: root_dir.to_string(),
        }
    }
}

#[async_trait]
impl CodeRetriever for DefaultCodeRetriever {
    async fn query_code(&self, query: &str, _top_k: usize) -> Result<Vec<CodeContext>> {
        // Return dummy/mock results for demonstration & tests
        Ok(vec![CodeContext {
            path: format!("{}/src/main.rs", self.root_dir),
            language: Language::Rust,
            snippet: "fn main() { println!(\"Hello, World!\"); }".to_string(),
            semantic_type: SemanticType::Function,
            dependencies: vec![],
            relevance_score: 0.95,
        }])
    }

    async fn get_code_by_location(&self, path: &str, identifier: &str) -> Result<CodeContext> {
        Ok(CodeContext {
            path: path.to_string(),
            language: Language::Rust,
            snippet: format!("fn {}() {{}}", identifier),
            semantic_type: SemanticType::Function,
            dependencies: vec![],
            relevance_score: 1.0,
        })
    }

    async fn find_callers(&self, path: &str, fn_name: &str) -> Result<Vec<CodeLocation>> {
        Ok(vec![CodeLocation {
            path: path.to_string(),
            start_line: 10,
            end_line: 12,
        }])
    }

    async fn analyze_change_impact(&self, path: &str, _old: &str, _new: &str) -> Result<ChangeImpact> {
        Ok(ChangeImpact {
            file_path: path.to_string(),
            affected_dependents: vec!["src/main.rs".to_string()],
            breaking_risk: "low".to_string(),
        })
    }
}
