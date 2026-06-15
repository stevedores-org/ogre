use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OgreExecutionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Safety violation: {reason}")]
    SafetyViolation { reason: String },

    #[error("Command failed: {reason}")]
    CommandFailed { reason: String },
}

pub type Result<T> = std::result::Result<T, OgreExecutionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub allowed_paths: Vec<String>,
    pub block_network: bool,
    pub timeout_seconds: u64,
}

pub struct SafeActionRunner {
    pub workspace_root: PathBuf,
    pub policy: ExecutionPolicy,
}

impl SafeActionRunner {
    pub fn new(workspace_root: &str, policy: ExecutionPolicy) -> Self {
        Self {
            workspace_root: PathBuf::from(workspace_root),
            policy,
        }
    }

    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        };

        // Normalize path by resolving components
        let mut normalized = PathBuf::new();
        for component in absolute_path.components() {
            match component {
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                std::path::Component::Normal(c) => {
                    normalized.push(c);
                }
                std::path::Component::CurDir => {}
                c => {
                    normalized.push(c.as_os_str());
                }
            }
        }

        // Canonicalize workspace root to compare
        let canonical_root = self.workspace_root.canonicalize().map_err(OgreExecutionError::Io)?;

        // Ensure normalized starts with canonical_root
        if !normalized.starts_with(&canonical_root) {
            return Err(OgreExecutionError::SafetyViolation {
                reason: format!("Access denied to path: {:?}", path),
            });
        }

        Ok(normalized)
    }


    pub fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let validated = self.validate_path(Path::new(path))?;
        std::fs::write(validated, content).map_err(OgreExecutionError::Io)?;
        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Result<String> {
        let validated = self.validate_path(Path::new(path))?;
        let content = std::fs::read_to_string(validated).map_err(OgreExecutionError::Io)?;
        Ok(content)
    }

    pub fn run_tool(&self, cmd: &str, args: &[&str]) -> Result<CommandResult> {
        // Run with a timeout, capture output
        let mut child = Command::new(cmd)
            .args(args)
            .current_dir(&self.workspace_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(OgreExecutionError::Io)?;

        // Simple block wait
        let output = child.wait_with_output().map_err(OgreExecutionError::Io)?;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        })
    }

    // Git Operations
    pub fn git_checkout_branch(&self, branch_name: &str) -> Result<()> {
        let res = self.run_tool("git", &["checkout", "-b", branch_name])?;
        if !res.success {
            // Try checkout without -b
            let res2 = self.run_tool("git", &["checkout", branch_name])?;
            if !res2.success {
                return Err(OgreExecutionError::CommandFailed {
                    reason: format!("Git checkout failed: {}", res2.stderr),
                });
            }
        }
        Ok(())
    }

    pub fn git_commit(&self, message: &str) -> Result<String> {
        self.run_tool("git", &["add", "."])?;
        let res = self.run_tool("git", &["commit", "-m", message])?;
        if !res.success {
            return Err(OgreExecutionError::CommandFailed {
                reason: format!("Git commit failed: {}", res.stderr),
            });
        }
        
        // Get last commit hash
        let hash_res = self.run_tool("git", &["rev-parse", "HEAD"])?;
        Ok(hash_res.stdout.trim().to_string())
    }

    pub fn git_diff(&self) -> Result<String> {
        let res = self.run_tool("git", &["diff"])?;
        Ok(res.stdout)
    }

    pub fn git_reset_hard(&self, target: &str) -> Result<()> {
        let res = self.run_tool("git", &["reset", "--hard", target])?;
        if !res.success {
            return Err(OgreExecutionError::CommandFailed {
                reason: format!("Git reset failed: {}", res.stderr),
            });
        }
        Ok(())
    }
}
