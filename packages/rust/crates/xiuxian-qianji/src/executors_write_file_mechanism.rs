use super::pathing::{resolve_destination_path, resolve_root_dir};
use super::template::render_template;
use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::path::Path;

/// Mechanism responsible for writing content to a local file path.
pub struct WriteFileMechanism {
    /// Destination path template (supports semantic placeholders and `{{key}}` interpolation).
    pub path: String,
    /// File content template (supports semantic placeholders and `{{key}}` interpolation).
    pub content: String,
    /// Context key used to store write metadata.
    pub output_key: String,
}

#[async_trait]
impl QianjiMechanism for WriteFileMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let resolved_path = render_template(&self.path, context)?;
        let resolved_content = render_template(&self.content, context)?;

        if resolved_path.trim().is_empty() {
            return Err("write_file path resolved to an empty value".to_string());
        }
        if resolved_path.contains("{{") || resolved_path.contains("}}") {
            return Err(format!(
                "write_file path contains unresolved template tokens: `{resolved_path}`"
            ));
        }

        let destination =
            resolve_destination_path(Path::new(resolved_path.as_str()), resolve_root_dir(context))?;

        fs::write(&destination, resolved_content.as_bytes()).map_err(|error| {
            format!(
                "write_file failed to write `{}`: {error}",
                destination.display()
            )
        })?;

        Ok(QianjiOutput {
            data: json!({
                self.output_key.clone(): {
                    "path": destination.display().to_string(),
                    "bytes_written": resolved_content.len()
                }
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}
