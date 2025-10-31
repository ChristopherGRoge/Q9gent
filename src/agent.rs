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
            "--verbose".to_string(), // Required for stream-json format
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

    /// Prepare platform-specific command for execution
    /// On Windows, .cmd and .bat files must be executed through cmd.exe
    #[cfg(target_os = "windows")]
    fn prepare_platform_command(&self, args: &[String]) -> (String, Vec<String>) {
        // Check if we're trying to execute a .cmd or .bat file
        let needs_cmd_wrapper = self.claude_path.to_lowercase().ends_with(".cmd")
            || self.claude_path.to_lowercase().ends_with(".bat");

        if needs_cmd_wrapper {
            debug!("🪟 Windows: Detected .cmd/.bat file, using cmd.exe wrapper");
            let mut cmd_args = vec!["/c".to_string(), self.claude_path.clone()];
            cmd_args.extend_from_slice(args);
            ("cmd.exe".to_string(), cmd_args)
        } else {
            debug!("🪟 Windows: Direct execution of {}", self.claude_path);
            (self.claude_path.clone(), args.to_vec())
        }
    }

    /// Prepare platform-specific command for execution
    /// On Unix-like systems, execute the command directly
    #[cfg(not(target_os = "windows"))]
    fn prepare_platform_command(&self, args: &[String]) -> (String, Vec<String>) {
        debug!("🐧 Unix: Direct execution of {}", self.claude_path);
        (self.claude_path.clone(), args.to_vec())
    }

    /// Spawn a claude process and return a channel to receive JSONL output
    pub async fn spawn(
        &self,
        request: AgentRequest,
    ) -> AppResult<(Child, mpsc::Receiver<AppResult<String>>)> {
        let args = self.build_command(&request);

        info!("🔨 Building Claude command - {} args", args.len());

        // Platform-specific command construction
        let (cmd_exe, cmd_args) = self.prepare_platform_command(&args);

        info!("📋 Executing: {} {:?}", cmd_exe, cmd_args);
        debug!("Full command: {} {}", cmd_exe, cmd_args.join(" "));

        let mut command = Command::new(&cmd_exe);
        command
            .args(&cmd_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        // On Windows, set environment variables to prevent Node.js buffering issues
        #[cfg(target_os = "windows")]
        {
            // Disable stdout buffering in Node.js to prevent EPIPE errors
            command.env("NODE_NO_WARNINGS", "1");
            // Force line-buffered output
            command.env("PYTHONUNBUFFERED", "1");
            debug!("🪟 Windows: Set Node.js environment variables to prevent buffering");
        }

        let mut child = command.spawn().map_err(|e| {
            warn!("❌ Failed to spawn Claude process: {}", e);
            warn!("   Command: {} {:?}", cmd_exe, cmd_args);
            AppError::ProcessSpawnFailed(format!(
                "Failed to spawn '{}' with args {:?}: {}",
                cmd_exe, cmd_args, e
            ))
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

            debug!("📖 Started stdout reader task");
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.is_empty() {
                    line_count += 1;
                    if line_count == 1 {
                        info!("📥 First line from Claude stdout");
                    }
                    debug!("Claude stdout line {}: {} chars", line_count, line.len());

                    // Try to send, but don't stop reading if channel is closed
                    // This prevents EPIPE errors on Windows when client disconnects
                    if tx_clone.send(Ok(line)).await.is_err() {
                        debug!("Channel closed, but continuing to drain stdout to prevent EPIPE");
                        // Continue reading to EOF to avoid breaking the pipe
                    }
                }
            }
            if line_count == 0 {
                warn!("⚠️  Stdout reader finished with ZERO lines read - process may have failed silently");
            } else {
                info!("📊 Stdout reader finished - {} lines read", line_count);
            }
        });

        // Spawn task to log stderr
        let tx_error = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            let mut stderr_count = 0;

            debug!("📖 Started stderr reader task");
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.is_empty() {
                    stderr_count += 1;
                    // Log at ERROR level for visibility
                    tracing::error!("🔴 Claude stderr [{}]: {}", stderr_count, line);

                    // If this looks like a critical error, send it to the output channel too
                    if line.contains("Error")
                        || line.contains("error")
                        || line.contains("failed")
                        || line.contains("Failed")
                        || line.contains("cannot")
                        || line.contains("Cannot")
                    {
                        let _ = tx_error
                            .send(Err(AppError::ProcessExecutionError(format!(
                                "Claude CLI error: {}",
                                line
                            ))))
                            .await;
                    }
                }
            }
            if stderr_count > 0 {
                warn!(
                    "⚠️  Stderr reader finished - {} error lines logged",
                    stderr_count
                );
            } else {
                debug!("Stderr reader finished - no errors");
            }
        });

        // Spawn task to monitor process exit status
        let pid_monitor = pid;
        let _tx_exit = tx.clone();
        tokio::spawn(async move {
            // Give the process a moment to start producing output
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            debug!(
                "⏰ Process monitor: Checking if PID {:?} is still alive",
                pid_monitor
            );
            // If we reach here and the channel is closed without sending data,
            // it means the process likely failed silently
        });

        Ok((child, rx))
    }

    /// Terminate a running process
    pub async fn terminate(mut child: Child) -> AppResult<()> {
        let pid = child.id();
        info!("🛑 Terminating Claude process - PID: {:?}", pid);
        child.kill().await.map_err(|e| {
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

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_cmd_wrapper() {
        let runner = AgentRunner::new("C:\\path\\to\\claude.cmd".to_string());
        let args = vec!["-p".to_string(), "test".to_string()];

        let (cmd, cmd_args) = runner.prepare_platform_command(&args);

        assert_eq!(cmd, "cmd.exe");
        assert_eq!(cmd_args[0], "/c");
        assert_eq!(cmd_args[1], "C:\\path\\to\\claude.cmd");
        assert_eq!(cmd_args[2], "-p");
        assert_eq!(cmd_args[3], "test");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_bat_wrapper() {
        let runner = AgentRunner::new("claude.BAT".to_string());
        let args = vec!["-p".to_string(), "test".to_string()];

        let (cmd, cmd_args) = runner.prepare_platform_command(&args);

        assert_eq!(cmd, "cmd.exe");
        assert_eq!(cmd_args[0], "/c");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_exe_direct() {
        let runner = AgentRunner::new("claude.exe".to_string());
        let args = vec!["-p".to_string(), "test".to_string()];

        let (cmd, cmd_args) = runner.prepare_platform_command(&args);

        assert_eq!(cmd, "claude.exe");
        assert_eq!(cmd_args[0], "-p");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_unix_direct_execution() {
        let runner = AgentRunner::new("/usr/bin/claude".to_string());
        let args = vec!["-p".to_string(), "test".to_string()];

        let (cmd, cmd_args) = runner.prepare_platform_command(&args);

        assert_eq!(cmd, "/usr/bin/claude");
        assert_eq!(cmd_args[0], "-p");
        assert_eq!(cmd_args[1], "test");
    }
}
