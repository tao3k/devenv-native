use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::scheduler_preflight::resolve_semantic_content;
use async_trait::async_trait;
use serde_json::{Value, json};

/// Mechanism responsible for one contract-validated CLI call.
pub struct CliCallMechanism {
    /// Stable contract id used for validation.
    pub contract: String,
    /// Authored argv vector.
    pub argv: Vec<String>,
    /// Context key used to merge the command result.
    pub output_key: String,
}

#[async_trait]
impl QianjiMechanism for CliCallMechanism {
    async fn execute(&self, context: &Value) -> Result<QianjiOutput, String> {
        let argv = self
            .argv
            .iter()
            .map(|token| resolve_semantic_content(token, context))
            .collect::<Result<Vec<_>, _>>()?;
        let Some(program) = argv.first() else {
            return Err("cli_call requires a non-empty argv".to_string());
        };
        let output = tokio::process::Command::new(program)
            .args(&argv[1..])
            .output()
            .await
            .map_err(|error| format!("failed to spawn `{program}`: {error}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or_default();
            return Err(format!(
                "CLI call `{}` failed with exit code {exit_code}: {stderr}",
                argv.join(" ")
            ));
        }

        Ok(QianjiOutput {
            data: json!({
                self.output_key.clone(): {
                    "contract": self.contract,
                    "transport": "cli",
                    "argv": argv,
                    "exit_code": output.status.code(),
                    "stdout": stdout,
                    "stderr": stderr
                }
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}
