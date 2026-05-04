#![cfg(test)]

use super::assets::MarkdownLintDiagnosticContractId;
use super::manifest::parse_manifest;

pub(super) fn generate_schema_json(
    contract_id: MarkdownLintDiagnosticContractId,
) -> anyhow::Result<String> {
    use schemars::schema_for;

    let manifest = parse_manifest(contract_id)?;
    let schema = match manifest.output.schema_provider.as_str() {
        "MarkdownLintReport" => {
            serde_json::to_string_pretty(&schema_for!(super::super::report::MarkdownLintReport))
                .map_err(|error| {
                    anyhow::anyhow!("failed to serialize MarkdownLintReport schema: {error}")
                })?
        }
        other => anyhow::bail!("unknown markdown lint schema provider `{other}`"),
    };
    Ok(format!("{schema}\n"))
}
