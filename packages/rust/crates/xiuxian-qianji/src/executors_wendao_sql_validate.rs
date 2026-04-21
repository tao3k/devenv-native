use async_trait::async_trait;
use serde_json::json;

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};

#[path = "executors_wendao_sql_validate/render_sql.rs"]
mod render_sql;
#[path = "executors_wendao_sql_validate/validation.rs"]
mod validation;

use super::contract::{SqlAuthorSpec, SqlFilter, SqlOrderTerm, SurfaceBundle, SurfaceColumn};
use super::input::required_context_string;
use super::render::validation_report_xml;
use validation::validate_and_render_sql;

/// Deterministic validation gate for XML-authored Wendao SQL specs.
pub struct WendaoSqlValidateMechanism {
    /// Context key containing the discovery bundle XML.
    pub surface_bundle_key: String,
    /// Context key containing the author output XML.
    pub author_spec_key: String,
    /// Output context key storing the canonical validated SQL.
    pub output_key: String,
    /// Output context key storing the validation report XML.
    pub report_key: String,
    /// Output context key storing rejection details.
    pub error_key: String,
    /// Branch label selected when validation succeeds.
    pub accepted_branch_label: Option<String>,
    /// Branch label selected when validation fails.
    pub rejected_branch_label: Option<String>,
}

#[async_trait]
impl QianjiMechanism for WendaoSqlValidateMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let surface_bundle_raw =
            required_context_string(context, self.surface_bundle_key.as_str())?;
        let author_spec_raw = required_context_string(context, self.author_spec_key.as_str())?;
        let bundle = super::contract::parse_surface_bundle_xml(surface_bundle_raw.as_str())?;
        let spec = super::contract::parse_sql_author_spec_xml(author_spec_raw.as_str())?;

        match validate_and_render_sql(&bundle, &spec) {
            Ok(canonical_sql) => Ok(QianjiOutput {
                data: json!({
                    self.output_key.clone(): canonical_sql.clone(),
                    self.report_key.clone(): validation_report_xml("accepted", "SQL author spec accepted", Some(canonical_sql.as_str())),
                }),
                instruction: branch_or_continue(self.accepted_branch_label.as_deref()),
            }),
            Err(message) => {
                let report = validation_report_xml("rejected", message.as_str(), None);
                if let Some(label) = self.rejected_branch_label.as_deref() {
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
