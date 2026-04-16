//! Seatbelt executor implementation
//!
//! Executes SBPL profiles on macOS via sandbox-exec.
//! This module reads pre-generated SBPL profiles and executes them.

use std::path::Path;
use tokio::process::Command as AsyncCommand;

use super::ExecutionResult;
use super::SandboxExecutor;
use super::execute_with_limits;

/// Seatbelt executor for macOS
#[derive(Debug, Clone)]
pub struct SeatbeltExecutor {
    default_timeout: u64,
}

impl SeatbeltExecutor {
    /// Create a new `SeatbeltExecutor` with a default timeout.
    #[must_use]
    pub fn new(default_timeout: u64) -> Self {
        Self { default_timeout }
    }

    /// Get executor name
    #[must_use]
    pub fn name(&self) -> &'static str {
        <Self as SandboxExecutor>::name(self)
    }
}

impl SeatbeltExecutor {
    /// Build sandbox-exec command from SBPL content
    fn build_command(profile_path: &Path, cmd_vec: &[String]) -> AsyncCommand {
        let mut cmd = AsyncCommand::new("sandbox-exec");
        cmd.arg("-f").arg(profile_path);

        if !cmd_vec.is_empty() {
            cmd.arg("--").args(cmd_vec);
        }

        cmd
    }
}

#[async_trait::async_trait]
impl SandboxExecutor for SeatbeltExecutor {
    fn name(&self) -> &'static str {
        "seatbelt"
    }

    async fn execute(&self, profile_path: &Path, input: &str) -> Result<ExecutionResult, String> {
        // Parse optional command from input
        let cmd_vec: Vec<String> = if input.is_empty() {
            vec!["/bin/pwd".to_string()]
        } else {
            // Input could be JSON with command
            match serde_json::from_str::<serde_json::Value>(input) {
                Ok(json) => {
                    if let Some(cmd_arr) = json.get("cmd").and_then(|c| c.as_array()) {
                        cmd_arr
                            .iter()
                            .filter_map(|c| c.as_str().map(String::from))
                            .collect()
                    } else {
                        vec!["/bin/pwd".to_string()]
                    }
                }
                Err(_) => {
                    // Treat input as shell command
                    vec!["/bin/bash".to_string(), "-c".to_string(), input.to_string()]
                }
            }
        };

        // Build and execute command
        let command = Self::build_command(profile_path, &cmd_vec);

        // Note: sandbox-exec on macOS doesn't support stdin input in the same way.
        execute_with_limits(command, self.default_timeout, 0).await
    }
}
