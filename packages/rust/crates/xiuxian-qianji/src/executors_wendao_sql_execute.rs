use async_trait::async_trait;
use serde_json::json;

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};

use super::gateway::query_sql_endpoint;
use super::input::{required_context_string, resolve_endpoint};
use super::render::execution_report_xml;

/// Deterministic execution node for validated Wendao SQL.
pub struct WendaoSqlExecuteMechanism {
    /// Context key containing canonical SQL to execute.
    pub sql_key: String,
    /// Output context key storing the stable SQL payload.
    pub output_key: String,
    /// Output context key storing the execution report XML.
    pub report_key: String,
    /// Output context key storing execution failures.
    pub error_key: String,
    /// Optional context key resolving the Wendao query endpoint.
    pub endpoint_key: Option<String>,
    /// Optional static Wendao query endpoint override.
    pub endpoint: Option<String>,
    /// Branch label selected when execution succeeds.
    pub success_branch_label: Option<String>,
    /// Branch label selected when execution fails.
    pub error_branch_label: Option<String>,
    /// Maximum preview rows included in the execution report.
    pub max_report_rows: usize,
}

#[async_trait]
impl QianjiMechanism for WendaoSqlExecuteMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let sql = required_context_string(context, self.sql_key.as_str())?;
        let endpoint = resolve_endpoint(
            context,
            self.endpoint.as_deref(),
            self.endpoint_key.as_deref(),
        )?;

        match query_sql_endpoint(endpoint.as_str(), sql.as_str()).await {
            Ok(payload) => Ok(QianjiOutput {
                data: json!({
                    self.output_key.clone(): serde_json::to_value(&payload)
                        .map_err(|error| format!("failed to serialize sql query payload: {error}"))?,
                    self.report_key.clone(): execution_report_xml(
                        "success",
                        "SQL execution succeeded",
                        Some(&payload),
                        self.max_report_rows,
                    ),
                }),
                instruction: branch_or_continue(self.success_branch_label.as_deref()),
            }),
            Err(message) => {
                let report =
                    execution_report_xml("error", message.as_str(), None, self.max_report_rows);
                if let Some(label) = self.error_branch_label.as_deref() {
                    Ok(QianjiOutput {
                        data: json!({
                            self.report_key.clone(): report,
                            self.error_key.clone(): message,
                        }),
                        instruction: FlowInstruction::SelectBranch(label.to_string()),
                    })
                } else {
                    Err(message)
                }
            }
        }
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

fn branch_or_continue(label: Option<&str>) -> FlowInstruction {
    if let Some(label) = label {
        FlowInstruction::SelectBranch(label.to_string())
    } else {
        FlowInstruction::Continue
    }
}
