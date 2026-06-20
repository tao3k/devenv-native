use std::collections::BTreeMap;

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use super::{
    julia_parser_file_summary_semantic_fingerprint,
    julia_parser_summary_allows_safe_incremental_file_for_repository,
};
use crate::julia_plugin_test_support::common::{
    ensure_linked_julia_parser_summary_service,
    skip_linked_julia_parser_summary_service_if_unavailable,
};
use crate::plugin::parser_summary::types::{
    JuliaParserDocAttachment, JuliaParserDocTargetKind, JuliaParserFileSummary, JuliaParserImport,
    JuliaParserSymbol, JuliaParserSymbolKind,
};

fn parser_summary_repository() -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
        ..RegisteredRepository::default()
    }
}

#[tokio::test]
async fn safe_incremental_live_service_distinguishes_leaf_and_root_files()
-> Result<(), Box<dyn std::error::Error>> {
    if skip_linked_julia_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_julia_parser_summary_service()?;
    let repository = parser_summary_repository();
    let leaf_repository = repository.clone();
    let leaf_is_safe = tokio::task::spawn_blocking(move || {
        julia_parser_summary_allows_safe_incremental_file_for_repository(
            &leaf_repository,
            "src/leaf.jl".into(),
            "alpha() = 2\nbeta() = 3\n",
        )
    })
    .await
    .unwrap_or_else(|error| panic!("blocking task should complete: {error}"))
    .unwrap_or_else(|error| panic!("safe incremental leaf file should decode: {error}"));
    let root_is_safe = tokio::task::spawn_blocking(move || {
        julia_parser_summary_allows_safe_incremental_file_for_repository(
            &repository,
            "src/FixturePkg.jl".into(),
            "module FixturePkg\ninclude(\"leaf.jl\")\nend\n",
        )
    })
    .await
    .unwrap_or_else(|error| panic!("blocking task should complete: {error}"))
    .unwrap_or_else(|error| panic!("root summary should decode: {error}"));

    assert!(
        leaf_is_safe,
        "leaf-only Julia file should stay incremental-safe"
    );
    assert!(
        !root_is_safe,
        "root Julia file should not stay incremental-safe"
    );
    Ok(())
}

#[test]
fn julia_parser_summary_file_semantic_fingerprint_changes_with_summary_semantics() {
    let base = JuliaParserFileSummary {
        module_name: None,
        exports: vec!["alpha".to_string()],
        imports: vec![JuliaParserImport {
            module: "Base".to_string(),
            reexported: false,
            dependency_kind: "using".to_string(),
            dependency_form: "module".to_string(),
            dependency_is_relative: false,
            dependency_relative_level: 0,
            dependency_local_name: None,
            dependency_parent: None,
            dependency_member: None,
            dependency_alias: None,
        }],
        symbols: vec![JuliaParserSymbol {
            name: "alpha".to_string(),
            kind: JuliaParserSymbolKind::Function,
            signature: Some("alpha()".to_string()),
            line_start: Some(1),
            line_end: Some(1),
            attributes: BTreeMap::default(),
        }],
        docstrings: vec![JuliaParserDocAttachment {
            target_name: "alpha".to_string(),
            target_kind: JuliaParserDocTargetKind::Symbol,
            target_path: Some("FixturePkg.alpha".to_string()),
            target_line_start: Some(1),
            target_line_end: Some(1),
            content: "alpha doc".to_string(),
        }],
        includes: vec![],
    };
    let same = base.clone();
    let mut changed = base.clone();
    changed.symbols[0].name = "beta".to_string();
    let mut relocated = base.clone();
    relocated.symbols[0].line_start = Some(10);
    relocated.symbols[0].line_end = Some(12);
    relocated.docstrings[0].target_line_start = Some(10);
    relocated.docstrings[0].target_line_end = Some(12);

    assert_eq!(
        julia_parser_file_summary_semantic_fingerprint(&base),
        julia_parser_file_summary_semantic_fingerprint(&same)
    );
    assert_eq!(
        julia_parser_file_summary_semantic_fingerprint(&base),
        julia_parser_file_summary_semantic_fingerprint(&relocated)
    );
    assert_ne!(
        julia_parser_file_summary_semantic_fingerprint(&base),
        julia_parser_file_summary_semantic_fingerprint(&changed)
    );
}
