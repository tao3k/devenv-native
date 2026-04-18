use super::{
    MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS, MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
    MarkdownLintDiagnosticContractSnapshot, generate_schema_json, generate_snapshot_contract_toml,
    markdown_lint_diagnostic_contract_assets, parse_manifest, schema_snapshot_path,
    snapshot_root_path,
};
use anyhow::Result;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),
        Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonicalize_json(value)))
                    .collect(),
            )
        }
        other => other,
    }
}

#[test]
fn markdown_lint_contract_assets_cover_the_checked_in_snapshot() {
    for contract_id in MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS {
        let assets = markdown_lint_diagnostic_contract_assets(contract_id)
            .unwrap_or_else(|| panic!("missing assets for `{contract_id}`"));
        let snapshot: MarkdownLintDiagnosticContractSnapshot = toml::from_str(assets.contract_toml)
            .unwrap_or_else(|error| panic!("invalid contract.toml for `{contract_id}`: {error}"));
        let schema: Value = serde_json::from_str(assets.schema_json)
            .unwrap_or_else(|error| panic!("invalid schema.json for `{contract_id}`: {error}"));

        assert_eq!(snapshot.id, *contract_id);
        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.task_types, vec!["cli_call", "diagnostic_render"]);
        assert_eq!(snapshot.cli.argv, vec!["wendao", "lint", "markdown"]);
        assert_eq!(snapshot.cli.positionals, vec!["paths"]);
        assert_eq!(
            snapshot.cli.flags,
            BTreeMap::from([
                ("output".to_string(), "--output".to_string()),
                ("root".to_string(), "--root".to_string()),
                ("skip_dirs".to_string(), "--skip-dir".to_string()),
            ])
        );
        assert_eq!(snapshot.output.format, "markdown_lint_report");
        assert_eq!(snapshot.output.schema, "schema.json");
        assert_eq!(snapshot.params.len(), 4);
        assert!(schema.get("properties").is_some());
    }
}

#[test]
fn markdown_lint_contract_snapshots_match_generated_contracts() -> Result<()> {
    for contract_id in MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS {
        let assets = markdown_lint_diagnostic_contract_assets(contract_id)
            .unwrap_or_else(|| panic!("missing assets"));
        assert_eq!(
            assets.contract_toml,
            generate_snapshot_contract_toml(contract_id)?,
            "contract snapshot drifted for `{contract_id}`"
        );
        assert_eq!(
            canonicalize_json(serde_json::from_str(assets.schema_json)?),
            canonicalize_json(serde_json::from_str(&generate_schema_json(contract_id)?)?),
            "schema snapshot drifted for `{contract_id}`"
        );
    }
    Ok(())
}

#[test]
fn markdown_lint_manifest_aligns_with_report_schema() -> Result<()> {
    for contract_id in MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS {
        let manifest = parse_manifest(contract_id)?;
        let schema: Value = serde_json::from_str(&generate_schema_json(contract_id)?)?;
        let properties = schema["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("report schema must expose properties"));
        let required = schema["required"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();

        assert_eq!(manifest.output.format, "markdown_lint_report");
        assert_eq!(manifest.output.schema_provider, "MarkdownLintReport");
        for required_field in ["checked_files", "files_with_issues", "issue_count", "files"] {
            assert!(
                properties.contains_key(required_field),
                "report schema missing `{required_field}` for `{contract_id}`"
            );
            assert!(
                required.contains(required_field),
                "report schema required set drifted for `{contract_id}` field `{required_field}`"
            );
        }
    }
    Ok(())
}

#[test]
fn markdown_lint_manifest_exposes_cli_contract() -> Result<()> {
    for contract_id in MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS {
        let manifest = parse_manifest(contract_id)?;
        assert_eq!(manifest.task_types, vec!["cli_call", "diagnostic_render"]);
        assert_eq!(manifest.cli.argv, vec!["wendao", "lint", "markdown"]);
        assert_eq!(manifest.cli.positionals, vec!["paths"]);
        assert_eq!(
            manifest.cli.flags,
            BTreeMap::from([
                ("output".to_string(), "--output".to_string()),
                ("root".to_string(), "--root".to_string()),
                ("skip_dirs".to_string(), "--skip-dir".to_string()),
            ])
        );
        assert_eq!(manifest.params.len(), 4);
    }
    Ok(())
}

#[test]
fn markdown_lint_contract_snapshot_directory_has_no_orphans() -> Result<()> {
    let expected = MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<BTreeSet<_>>();
    let actual = std::fs::read_dir(snapshot_root_path())?
        .map(|entry| -> Result<String> {
            let entry = entry?;
            let file_type = entry.file_type()?;
            anyhow::ensure!(
                file_type.is_dir(),
                "unexpected non-directory snapshot entry"
            );
            Ok(entry.file_name().to_string_lossy().into_owned())
        })
        .collect::<Result<BTreeSet<_>>>()?;

    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn markdown_lint_contract_paths_exist_on_disk() {
    for contract_id in MARKDOWN_LINT_DIAGNOSTIC_CONTRACT_IDS {
        assert!(
            Path::new(&super::contract_snapshot_path(contract_id)).exists(),
            "missing contract snapshot path for `{contract_id}`",
        );
        assert!(
            Path::new(&schema_snapshot_path(contract_id)).exists(),
            "missing schema snapshot path for `{contract_id}`",
        );
    }
}

#[test]
fn markdown_lint_contract_snapshot_id_stays_stable() {
    assert_eq!(
        MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
        "wendao.markdown_lint.diagnostics"
    );
}
