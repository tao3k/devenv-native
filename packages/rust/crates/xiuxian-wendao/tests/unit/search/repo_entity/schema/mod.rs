use std::collections::BTreeMap;

use xiuxian_db_store::{
    LanceArray, LanceListArray, LanceRecordBatch, LanceStringArray, LanceUInt32Array,
};

use super::batches::repo_entity_batches;
use super::definitions::{
    COLUMN_ATTRIBUTES_JSON, COLUMN_LINE_END, COLUMN_LINE_START, COLUMN_MODULE_ID,
    COLUMN_PROJECTION_PAGE_IDS,
};
use super::helpers::{
    infer_code_language, repo_navigation_target, serialize_backlink_items_json,
    serialize_symbol_attributes_json,
};
use super::rows::rows_from_analysis;
use super::{
    entity_kind_column, hit_json_column, id_column, language_column, path_column,
    projected_columns, search_text_column, symbol_kind_column,
};
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::{
    DocRecord, ExampleRecord, ModuleRecord, RelationKind, RelationRecord, RepoSymbolKind,
    RepositoryRecord, SymbolRecord,
};
use crate::gateway::studio::types::{SearchBacklinkItem, SearchHit};

#[test]
fn projected_columns_and_accessors_match_the_schema_layout() {
    assert_eq!(
        projected_columns(),
        [
            "id",
            "entity_kind",
            "name",
            "name_folded",
            "qualified_name_folded",
            "path",
            "path_folded",
            "language",
            "symbol_kind",
            "signature_folded",
            "summary_folded",
            "related_symbols_folded",
            "related_modules_folded",
            "saliency_score",
        ]
    );
    assert_eq!(id_column(), "id");
    assert_eq!(entity_kind_column(), "entity_kind");
    assert_eq!(hit_json_column(), "hit_json");
    assert_eq!(language_column(), "language");
    assert_eq!(path_column(), "path");
    assert_eq!(search_text_column(), "search_text");
    assert_eq!(symbol_kind_column(), "symbol_kind");
}

#[test]
fn helper_serialization_uses_optional_metadata_shapes() {
    let mut attributes = BTreeMap::new();
    attributes.insert("kind".to_string(), "function".to_string());

    assert_eq!(
        ok_or_panic(
            serialize_symbol_attributes_json(&attributes),
            "serialize symbol attributes"
        ),
        Some(r#"{"kind":"function"}"#.to_string())
    );
    assert_eq!(
        ok_or_panic(
            serialize_symbol_attributes_json(&BTreeMap::new()),
            "serialize empty symbol attributes"
        ),
        None
    );

    let backlink_items = vec![SearchBacklinkItem {
        id: "backlink:1".to_string(),
        title: Some("Backlink".to_string()),
        path: Some("src/lib.rs".to_string()),
        kind: Some("documents".to_string()),
    }];

    assert_eq!(
        ok_or_panic(
            serialize_backlink_items_json(Some(&backlink_items)),
            "serialize backlink items"
        ),
        Some(
            r#"[{"id":"backlink:1","title":"Backlink","path":"src/lib.rs","kind":"documents"}]"#
                .to_string()
        )
    );
    assert_eq!(
        ok_or_panic(
            serialize_backlink_items_json(None),
            "serialize no backlink items"
        ),
        None
    );
}

#[test]
fn code_language_and_navigation_helpers_normalize_inputs() {
    assert_eq!(
        infer_code_language("src/Example.JL"),
        Some("julia".to_string())
    );
    assert_eq!(
        infer_code_language("src/Example.tsx"),
        Some("typescript".to_string())
    );
    assert_eq!(infer_code_language("src/Example.txt"), None);

    let target = repo_navigation_target("alpha/repo", r"src\Example.jl", Some(12), Some(18));
    assert_eq!(target.path, "alpha/repo/src/Example.jl");
    assert_eq!(target.category, "repo_code");
    assert_eq!(target.project_name.as_deref(), Some("alpha/repo"));
    assert_eq!(target.root_label.as_deref(), Some("alpha/repo"));
    assert_eq!(target.line, Some(12));
    assert_eq!(target.line_end, Some(18));
    assert_eq!(target.column, None);
}

#[test]
fn rows_from_analysis_preserve_structured_symbol_payload_fields() {
    let rows = ok_or_panic(
        rows_from_analysis("BaseModelica", &sample_analysis()),
        "rows from analysis",
    );
    let symbol_row = some_or_panic(
        rows.iter()
            .find(|row| row.id == "symbol:BaseModelica.solve"),
        "symbol row",
    );

    assert_eq!(symbol_row.entity_kind, "symbol");
    assert_eq!(symbol_row.module_id.as_deref(), Some("module:BaseModelica"));
    assert_eq!(symbol_row.signature.as_deref(), Some("solve()"));
    assert_eq!(symbol_row.line_start, Some(7));
    assert_eq!(symbol_row.line_end, Some(9));
    assert_eq!(symbol_row.audit_status.as_deref(), Some("verified"));
    assert_eq!(symbol_row.verification_state.as_deref(), Some("verified"));
    assert_eq!(
        symbol_row.attributes_json.as_deref(),
        Some(r#"{"arity":"0"}"#)
    );
    assert_eq!(
        symbol_row.hierarchy,
        vec!["src".to_string(), "BaseModelica.jl".to_string()]
    );
    assert!(symbol_row.projection_page_ids.contains(
        &"repo:BaseModelica:projection:reference:symbol:symbol:BaseModelica.solve".to_string()
    ));
}

#[test]
fn rows_from_analysis_builds_example_rows_with_projected_hit_payload() {
    let rows = ok_or_panic(
        rows_from_analysis("BaseModelica", &sample_analysis()),
        "rows from analysis",
    );
    let example_row = some_or_panic(
        rows.iter()
            .find(|row| row.id == "example:BaseModelica.solve"),
        "example row",
    );
    let hit: SearchHit = ok_or_panic(
        serde_json::from_str(example_row.hit_json.as_str()),
        "deserialize example hit payload",
    );

    assert_eq!(example_row.entity_kind, "example");
    assert_eq!(example_row.line_start, Some(1));
    assert_eq!(example_row.line_end, None);
    assert_eq!(
        example_row.hierarchical_uri.as_deref(),
        Some(
            "wendao://repo/msl/BaseModelica/examples/examples:solve.jl/example:BaseModelica.solve"
        )
    );
    assert_eq!(hit.doc_type.as_deref(), Some("example"));
    assert_eq!(hit.match_reason.as_deref(), Some("repo_example_search"));
    assert_eq!(
        hit.navigation_target
            .as_ref()
            .map(|target| target.path.as_str()),
        Some("BaseModelica/examples/solve.jl")
    );
}

#[test]
fn repo_entity_batches_encode_structured_columns() {
    let rows = ok_or_panic(
        rows_from_analysis("BaseModelica", &sample_analysis()),
        "rows from analysis",
    );
    let batches = ok_or_panic(repo_entity_batches(&rows), "repo entity batches");
    let batch = &batches[0];
    let row_index = some_or_panic(
        rows.iter()
            .position(|row| row.id == "symbol:BaseModelica.solve"),
        "symbol row index",
    );

    let module_id = string_column(batch, COLUMN_MODULE_ID);
    let attributes_json = string_column(batch, COLUMN_ATTRIBUTES_JSON);
    let line_start = uint32_column(batch, COLUMN_LINE_START);
    let line_end = uint32_column(batch, COLUMN_LINE_END);
    let projection_page_ids = utf8_list_values(batch, COLUMN_PROJECTION_PAGE_IDS, row_index);

    assert_eq!(module_id.value(row_index), "module:BaseModelica");
    assert_eq!(attributes_json.value(row_index), r#"{"arity":"0"}"#);
    assert_eq!(line_start.value(row_index), 7);
    assert_eq!(line_end.value(row_index), 9);
    assert!(projection_page_ids.contains(
        &"repo:BaseModelica:projection:reference:symbol:symbol:BaseModelica.solve".to_string()
    ));
}

fn ok_or_panic<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn some_or_panic<T>(value: Option<T>, context: &str) -> T {
    let Some(value) = value else {
        panic!("{context}");
    };
    value
}

fn sample_analysis() -> RepositoryAnalysisOutput {
    let repo_id = "BaseModelica".to_string();
    let module_id = "module:BaseModelica".to_string();
    let symbol_id = "symbol:BaseModelica.solve".to_string();
    let example_id = "example:BaseModelica.solve".to_string();
    let doc_id = "doc:BaseModelica.solve".to_string();
    let mut attributes = BTreeMap::new();
    attributes.insert("arity".to_string(), "0".to_string());

    RepositoryAnalysisOutput {
        repository: Some(RepositoryRecord {
            repo_id: repo_id.clone(),
            name: repo_id.clone(),
            path: ".".to_string(),
            ..RepositoryRecord::default()
        }),
        modules: vec![ModuleRecord {
            repo_id: repo_id.clone(),
            module_id: module_id.clone(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.clone(),
            symbol_id: symbol_id.clone(),
            module_id: Some(module_id.clone()),
            name: "solve".to_string(),
            qualified_name: "BaseModelica.solve".to_string(),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some("solve()".to_string()),
            audit_status: Some("verified".to_string()),
            verification_state: Some("verified".to_string()),
            attributes,
        }],
        imports: Vec::new(),
        examples: vec![ExampleRecord {
            repo_id: repo_id.clone(),
            example_id: example_id.clone(),
            title: "Solve Example".to_string(),
            path: "examples/solve.jl".to_string(),
            summary: Some("Solve a base model".to_string()),
        }],
        docs: vec![DocRecord {
            repo_id: repo_id.clone(),
            doc_id: doc_id.clone(),
            title: "Solve Documentation".to_string(),
            path: "docs/solve.md".to_string(),
            format: Some("markdown".to_string()),
            doc_target: None,
        }],
        relations: vec![
            RelationRecord {
                repo_id: repo_id.clone(),
                source_id: doc_id,
                target_id: symbol_id.clone(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id,
                source_id: example_id,
                target_id: symbol_id,
                kind: RelationKind::ExampleOf,
            },
        ],
        diagnostics: Vec::new(),
    }
}

fn string_column<'a>(batch: &'a LanceRecordBatch, name: &str) -> &'a LanceStringArray {
    let array = some_or_panic(batch.column_by_name(name), "missing string column");
    let Some(strings) = array.as_any().downcast_ref::<LanceStringArray>() else {
        panic!("column should be utf8: {name}");
    };
    strings
}

fn uint32_column<'a>(batch: &'a LanceRecordBatch, name: &str) -> &'a LanceUInt32Array {
    let array = some_or_panic(batch.column_by_name(name), "missing uint32 column");
    let Some(values) = array.as_any().downcast_ref::<LanceUInt32Array>() else {
        panic!("column should be uint32: {name}");
    };
    values
}

fn utf8_list_values(batch: &LanceRecordBatch, name: &str, row: usize) -> Vec<String> {
    let array = some_or_panic(batch.column_by_name(name), "missing list column");
    let Some(list) = array.as_any().downcast_ref::<LanceListArray>() else {
        panic!("column should be list: {name}");
    };
    let values = list.value(row);
    let Some(strings) = values.as_any().downcast_ref::<LanceStringArray>() else {
        panic!("list values should be utf8: {name}");
    };
    (0..strings.len())
        .map(|index| strings.value(index).to_string())
        .collect()
}
