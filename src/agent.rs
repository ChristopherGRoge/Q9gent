use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::error::{AppError, AppResult};

/// Agent spawn request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    /// Type of agent (informational, not used by claude CLI)
    pub agent_type: String,

    /// The prompt to send to the agent
    pub prompt: String,

    /// Additional CLI flags as key-value pairs
    #[serde(default)]
    pub flags: Vec<String>,

    /// Allowed tools for the agent (--allowedTools flag)
    #[serde(default)]
    pub tools_allowed: Vec<String>,

    /// System prompt to append (--append-system-prompt flag)
    pub system_append: Option<String>,

    /// Resume session ID (--resume flag)
    pub resume_id: Option<String>,
}

/// Agent runner - spawns and manages claude CLI processes
pub struct AgentRunner {
    claude_path: String,
}

impl AgentRunner {
    pub fn new(claude_path: String) -> Self {
        Self { claude_path }
    }

    /// Build the command line arguments for claude
    fn build_command(&self, request: &AgentRequest) -> Vec<String> {
        let mut args = vec![
            "-p".to_string(),
            request.prompt.clone(),
            "--output-format".to_string(),
            "stream-json".to_string(),
        ];

        // Add allowed tools
        if !request.tools_allowed.is_empty() {
            args.push("--allowedTools".to_string());
            args.push(request.tools_allowed.join(","));
        }

        // Add system prompt append
        if let Some(ref system_prompt) = request.system_append {
            args.push("--append-system-prompt".to_string());
            args.push(system_prompt.clone());
        }

        // Add resume session ID
        if let Some(ref resume_id) = request.resume_id {
            args.push("--resume".to_string());
            args.push(resume_id.clone());
        }

        // Add any additional flags
        args.extend(request.flags.clone());

        args
    }

    /// Spawn a claude process and return a channel to receive JSONL output
    pub async fn spawn(
        &self,
        request: AgentRequest,
    ) -> AppResult<(Child, mpsc::Receiver<AppResult<String>>)> {
        let args = self.build_command(&request);

        info!("🔨 Building Claude command - {} args", args.len());
        debug!("Command: {} {:?}", self.claude_path, args);

        let mut child = Command::new(&self.claude_path)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                warn!("❌ Failed to spawn Claude process: {}", e);
                AppError::ProcessSpawnFailed(e.to_string())
            })?;

        let pid = child.id();
        info!("✓ Claude process spawned - PID: {:?}", pid);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::ProcessSpawnFailed("Failed to capture stdout".to_string()))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::ProcessSpawnFailed("Failed to capture stderr".to_string()))?;

        let (tx, rx) = mpsc::channel(100);

        // Spawn task to read stdout line-by-line
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut line_count = 0;

            while let Ok(Some(line)) = lines.next_line().await {
                if !line.is_empty() {
                    line_count += 1;
                    if line_count == 1 {
                        debug!("📥 First line from Claude stdout");
                    }
                    debug!("Claude stdout line {}: {} chars", line_count, line.len());
                    if tx_clone.send(Ok(line)).await.is_err() {
                        debug!("Channel closed, stopping stdout reader");
                        break;
                    }
                }
            }
            debug!("📊 Stdout reader finished - {} lines read", line_count);
        });

        // Spawn task to log stderr
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if !line.is_empty() {
                    warn!("⚠️  Claude stderr: {}", line);
                }
            }
            debug!("Stderr reader finished");
        });

        Ok((child, rx))
    }

    /// Terminate a running process
    pub async fn terminate(mut child: Child) -> AppResult<()> {
        let pid = child.id();
        info!("🛑 Terminating Claude process - PID: {:?}", pid);
        child
            .kill()
            .await
            .map_err(|e| {
                warn!("Failed to kill process: {}", e);
                AppError::ProcessExecutionError(e.to_string())
            })?;
        info!("✓ Process terminated");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command() {
        let runner = AgentRunner::new("claude".to_string());

        let request = AgentRequest {
            agent_type: "test".to_string(),
            prompt: "Hello world".to_string(),
            flags: vec![],
            tools_allowed: vec!["read_file".to_string(), "write_file".to_string()],
            system_append: Some("You are a test agent".to_string()),
            resume_id: Some("session-123".to_string()),
        };

        let args = runner.build_command(&request);

        assert!(args.contains(&"-p".to_string()));
        assert!(args.contains(&"Hello world".to_string()));
        assert!(args.contains(&"--output-format".to_string()));
        assert!(args.contains(&"stream-json".to_string()));
        assert!(args.contains(&"--allowedTools".to_string()));
        assert!(args.contains(&"read_file,write_file".to_string()));
        assert!(args.contains(&"--append-system-prompt".to_string()));
        assert!(args.contains(&"You are a test agent".to_string()));
        assert!(args.contains(&"--resume".to_string()));
        assert!(args.contains(&"session-123".to_string()));
    }
}
