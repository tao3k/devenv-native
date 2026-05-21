use std::collections::BTreeMap;

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{DocRecord, ModuleRecord, RelationKind};

use super::{
    build_incremental_doc_relations, doc_targets_for_annotation_doc, doc_targets_for_file_doc,
};

#[test]
fn doc_targets_for_file_doc_links_users_guide_docs_to_owner_modules() {
    let module_lookup = BTreeMap::from([
        (
            "DemoLib".to_string(),
            module("repo:modelica-demo:module:DemoLib", "DemoLib", "package.mo"),
        ),
        (
            "DemoLib.UsersGuide".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.UsersGuide",
                "DemoLib.UsersGuide",
                "UsersGuide/package.mo",
            ),
        ),
        (
            "DemoLib.Controllers".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.Controllers",
                "DemoLib.Controllers",
                "Controllers/package.mo",
            ),
        ),
        (
            "DemoLib.Controllers.UsersGuide".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.Controllers.UsersGuide",
                "DemoLib.Controllers.UsersGuide",
                "Controllers/UsersGuide/package.mo",
            ),
        ),
        (
            "DemoLib.Controllers.UsersGuide.Tutorial".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.Controllers.UsersGuide.Tutorial",
                "DemoLib.Controllers.UsersGuide.Tutorial",
                "Controllers/UsersGuide/Tutorial/package.mo",
            ),
        ),
    ]);
    let payload = json!([
        {
            "path": "README.md",
            "targets": doc_targets_for_file_doc(
                "README.md",
                "DemoLib",
                &module_lookup,
                Some("repo:modelica-demo:module:DemoLib"),
            ),
        },
        {
            "path": "UsersGuide/Overview.mo",
            "targets": doc_targets_for_file_doc(
                "UsersGuide/Overview.mo",
                "DemoLib",
                &module_lookup,
                Some("repo:modelica-demo:module:DemoLib"),
            ),
        },
        {
            "path": "Controllers/UsersGuide/Tuning.mo",
            "targets": doc_targets_for_file_doc(
                "Controllers/UsersGuide/Tuning.mo",
                "DemoLib",
                &module_lookup,
                Some("repo:modelica-demo:module:DemoLib"),
            ),
        },
        {
            "path": "Controllers/UsersGuide/Tutorial/FirstSteps.mo",
            "targets": doc_targets_for_file_doc(
                "Controllers/UsersGuide/Tutorial/FirstSteps.mo",
                "DemoLib",
                &module_lookup,
                Some("repo:modelica-demo:module:DemoLib"),
            ),
        },
    ]);

    insta::assert_json_snapshot!(
        "doc_targets_for_file_doc_links_users_guide_docs_to_owner_modules",
        payload
    );
}

#[test]
fn doc_targets_for_annotation_doc_links_users_guide_docs_to_owner_modules() {
    let module_lookup = BTreeMap::from([
        (
            "DemoLib".to_string(),
            module("repo:modelica-demo:module:DemoLib", "DemoLib", "package.mo"),
        ),
        (
            "DemoLib.UsersGuide".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.UsersGuide",
                "DemoLib.UsersGuide",
                "UsersGuide/package.mo",
            ),
        ),
        (
            "DemoLib.Controllers".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.Controllers",
                "DemoLib.Controllers",
                "Controllers/package.mo",
            ),
        ),
        (
            "DemoLib.Controllers.UsersGuide".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.Controllers.UsersGuide",
                "DemoLib.Controllers.UsersGuide",
                "Controllers/UsersGuide/package.mo",
            ),
        ),
        (
            "DemoLib.Controllers.UsersGuide.Tutorial".to_string(),
            module(
                "repo:modelica-demo:module:DemoLib.Controllers.UsersGuide.Tutorial",
                "DemoLib.Controllers.UsersGuide.Tutorial",
                "Controllers/UsersGuide/Tutorial/package.mo",
            ),
        ),
    ]);
    let payload = json!([
        {
            "path": "UsersGuide/Overview.mo",
            "targets": doc_targets_for_annotation_doc(
                "UsersGuide/Overview.mo",
                "DemoLib",
                &module_lookup,
                &[],
                Some("repo:modelica-demo:module:DemoLib"),
            ),
        },
        {
            "path": "Controllers/UsersGuide/Tuning.mo",
            "targets": doc_targets_for_annotation_doc(
                "Controllers/UsersGuide/Tuning.mo",
                "DemoLib",
                &module_lookup,
                &[],
                Some("repo:modelica-demo:module:DemoLib"),
            ),
        },
        {
            "path": "Controllers/UsersGuide/Tutorial/FirstSteps.mo",
            "targets": doc_targets_for_annotation_doc(
                "Controllers/UsersGuide/Tutorial/FirstSteps.mo",
                "DemoLib",
                &module_lookup,
                &[],
                Some("repo:modelica-demo:module:DemoLib"),
            ),
        },
    ]);

    insta::assert_json_snapshot!(
        "doc_targets_for_annotation_doc_links_users_guide_docs_to_owner_modules",
        payload
    );
}

#[test]
fn build_incremental_doc_relations_rehydrates_readme_and_annotation_docs() {
    let modules = vec![
        module("repo:modelica-demo:module:DemoLib", "DemoLib", "package.mo"),
        module(
            "repo:modelica-demo:module:DemoLib.Controllers",
            "DemoLib.Controllers",
            "Controllers/package.mo",
        ),
    ];
    let docs = vec![
        DocRecord {
            repo_id: "modelica-demo".to_string().into(),
            doc_id: "repo:modelica-demo:doc:README.md".to_string().into(),
            title: "README".to_string(),
            path: "README.md".to_string().into(),
            format: Some("md".to_string()),
            doc_target: None,
        },
        DocRecord {
            repo_id: "modelica-demo".to_string().into(),
            doc_id: "repo:modelica-demo:doc:Controllers/package.mo#annotation.documentation"
                .to_string()
                .into(),
            title: "Controllers documentation".to_string(),
            path: "Controllers/package.mo#annotation.documentation"
                .to_string()
                .into(),
            format: Some("modelica_annotation".to_string()),
            doc_target: None,
        },
    ];

    let relations = build_incremental_doc_relations("modelica-demo", &modules, &[], &docs);
    let payload = json!(
        relations
            .into_iter()
            .map(|relation| {
                (
                    relation.source_id,
                    relation.target_id,
                    relation.kind == RelationKind::Documents,
                )
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        payload,
        json!([
            [
                "repo:modelica-demo:doc:README.md",
                "repo:modelica-demo:module:DemoLib",
                true
            ],
            [
                "repo:modelica-demo:doc:Controllers/package.mo#annotation.documentation",
                "repo:modelica-demo:module:DemoLib.Controllers",
                true
            ]
        ])
    );
}

fn module(module_id: &str, qualified_name: &str, path: &str) -> ModuleRecord {
    ModuleRecord {
        repo_id: "modelica-demo".to_string().into(),
        module_id: module_id.to_string().into(),
        qualified_name: qualified_name.to_string(),
        path: path.to_string().into(),
    }
}
