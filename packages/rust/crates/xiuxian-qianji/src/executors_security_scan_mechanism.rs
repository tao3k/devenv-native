//! AST-based Security Scanning Mechanism.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::executors::security_scan::input::{
    collect_file_paths, resolve_base_dir, resolve_scan_path,
};
use async_trait::async_trait;
use serde_json::json;
use std::fs;
use xiuxian_ast::SecurityScanner;

/// Mechanism responsible for statically analyzing code files for security violations.
pub struct SecurityScanMechanism {
    /// Context key containing a list of file paths to scan.
    pub files_key: String,
    /// Context key to output the list of violations.
    pub output_key: String,
    /// Whether to abort execution if any violation is found.
    pub abort_on_violation: bool,
    /// Context key for the working directory to resolve relative paths against.
    pub cwd_key: Option<String>,
}

#[async_trait]
impl QianjiMechanism for SecurityScanMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let file_paths = collect_file_paths(context, &self.files_key)?;
        let base_dir = resolve_base_dir(context, self.cwd_key.as_ref());

        let mut all_violations = Vec::new();
        let scanner = SecurityScanner::new();

        for file_str in file_paths {
            let path_buf = resolve_scan_path(&file_str, base_dir);

            // Read file if it exists (it might be a deleted staged file, so we skip reading errors softly)
            if path_buf.exists()
                && path_buf.is_file()
                && let Ok(content) = fs::read_to_string(&path_buf)
            {
                let file_violations = scanner.scan_all(&content);
                for v in file_violations {
                    all_violations.push(json!({
                        "file": file_str,
                        "rule_id": v.rule_id,
                        "description": v.description,
                        "line": v.line,
                        "snippet": v.snippet,
                    }));
                }
            }
        }

        if !all_violations.is_empty() && self.abort_on_violation {
            return Ok(QianjiOutput {
                data: json!({ self.output_key.clone(): all_violations }),
                instruction: FlowInstruction::Abort("security_violation".to_string()),
            });
        }

        Ok(QianjiOutput {
            data: json!({ self.output_key.clone(): all_violations }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}
