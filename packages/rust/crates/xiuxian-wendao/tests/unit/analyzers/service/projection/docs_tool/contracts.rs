use super::{
    DOCS_CONTRACT_IDS, DOCS_DOCUMENT_CONTRACT_ID, DOCS_NAVIGATION_CONTRACT_ID,
    DOCS_PAGE_INDEX_TREE_CONTRACT_ID, DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID, DOCS_SEARCH_CONTRACT_ID,
    DocsCapabilityContractSnapshot, docs_capability_contract_assets, generate_schema_json,
    generate_snapshot_contract_toml, parse_manifest, schema_snapshot_path, snapshot_root_path,
};
use anyhow::Result;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn docs_contract_assets_cover_declared_docs_contracts() {
    for contract_id in DOCS_CONTRACT_IDS {
        let assets = docs_capability_contract_assets(contract_id)
            .unwrap_or_else(|| panic!("missing assets for `{contract_id}`"));
        let snapshot: DocsCapabilityContractSnapshot = toml::from_str(assets.contract_toml)
            .unwrap_or_else(|error| panic!("invalid contract.toml for `{contract_id}`: {error}"));
        let schema: Value = serde_json::from_str(assets.schema_json)
            .unwrap_or_else(|error| panic!("invalid schema.json for `{contract_id}`: {error}"));

        assert_eq!(snapshot.id, *contract_id);
        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.task_types, vec!["http_call", "cli_call"]);
        assert!(schema.get("properties").is_some());
    }
}

#[test]
fn docs_contract_snapshots_match_generated_contracts() -> Result<()> {
    for contract_id in DOCS_CONTRACT_IDS {
        let assets = docs_capability_contract_assets(contract_id)
            .unwrap_or_else(|| panic!("missing assets"));
        assert_eq!(
            assets.contract_toml,
            generate_snapshot_contract_toml(contract_id)?,
            "contract snapshot drifted for `{contract_id}`"
        );
        let snapshot_schema: Value = serde_json::from_str(assets.schema_json)?;
        let generated_schema: Value = serde_json::from_str(&generate_schema_json(contract_id)?)?;
        assert_eq!(
            snapshot_schema, generated_schema,
            "schema snapshot drifted for `{contract_id}`"
        );
    }
    Ok(())
}

#[test]
fn docs_contract_manifests_align_with_tool_schemas() -> Result<()> {
    for contract_id in DOCS_CONTRACT_IDS {
        let manifest = parse_manifest(contract_id)?;
        let schema: Value = serde_json::from_str(&generate_schema_json(contract_id)?)?;
        let properties = schema["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("tool schema must expose properties"));
        let required = schema["required"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        let runtime_injected = manifest
            .tool
            .runtime_injected
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();

        for param in manifest.params {
            if runtime_injected.contains(&param.name) {
                assert!(
                    !properties.contains_key(param.name.as_str()),
                    "runtime injected param `{}` must stay out of schema for `{contract_id}`",
                    param.name
                );
                continue;
            }

            assert!(
                properties.contains_key(param.name.as_str()),
                "schema missing param `{}` for `{contract_id}`",
                param.name
            );
            assert_eq!(
                required.contains(&param.name),
                param.required,
                "required set drifted for `{contract_id}` param `{}`",
                param.name
            );
        }
    }
    Ok(())
}

#[test]
fn docs_contract_snapshot_directory_has_no_orphans() -> Result<()> {
    let expected = DOCS_CONTRACT_IDS
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
fn docs_contract_paths_exist_on_disk() {
    for contract_id in DOCS_CONTRACT_IDS {
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
fn docs_contract_snapshot_ids_stay_stable() {
    assert_eq!(DOCS_SEARCH_CONTRACT_ID, "wendao.docs.search");
    assert_eq!(DOCS_DOCUMENT_CONTRACT_ID, "wendao.docs.document");
    assert_eq!(
        DOCS_PAGE_INDEX_TREE_CONTRACT_ID,
        "wendao.docs.page_index_tree"
    );
    assert_eq!(DOCS_NAVIGATION_CONTRACT_ID, "wendao.docs.navigation");
    assert_eq!(
        DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID,
        "wendao.docs.retrieval_context"
    );
}
