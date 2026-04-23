//! Shell command execution mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::scheduler_preflight::resolve_semantic_content;
use async_trait::async_trait;
use serde_json::json;

/// Mechanism responsible for executing local shell commands.
pub struct ShellMechanism {
    /// The shell command to execute.
    pub cmd: String,
    /// Whether to continue if the command returns a non-zero exit code.
    pub allow_fail: bool,
    /// Whether to abort the workflow if stdout is empty.
    pub stop_on_empty_stdout: bool,
    /// Message to emit if stopping due to empty stdout.
    pub empty_reason: Option<String>,
    /// The context key to store the stdout result.
    pub output_key: String,
}

#[async_trait]
impl QianjiMechanism for ShellMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        // Simple string interpolation from context (e.g., {{message}})
        let mut final_cmd = resolve_semantic_content(&self.cmd, context)?;
        if let Some(obj) = context.as_object() {
            for (k, v) in obj {
                if let Some(v_str) = v.as_str() {
                    let placeholder = format!("{{{{{k}}}}}");
                    final_cmd = final_cmd.replace(&placeholder, v_str);
                }
            }
        }

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&final_cmd)
            .output()
            .await
            .map_err(|e| format!("Failed to spawn shell: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() && !self.allow_fail {
            return Err(format!("Command failed: {final_cmd}\nStderr: {stderr}"));
        }

        if self.stop_on_empty_stdout && stdout.is_empty() {
            return Ok(QianjiOutput {
                data: json!({}),
                instruction: FlowInstruction::Abort(
                    self.empty_reason
                        .clone()
                        .unwrap_or_else(|| "Empty stdout".to_string()),
                ),
            });
        }

        let mut data = serde_json::Map::new();
        data.insert(self.output_key.clone(), serde_json::Value::String(stdout));

        Ok(QianjiOutput {
            data: serde_json::Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}
