use std::collections::BTreeMap;

use crate::modelica_plugin::parser_summary::incremental::{
    modelica_file_semantic_fingerprint, modelica_parser_file_summary_semantic_fingerprint,
};
use crate::modelica_plugin::parser_summary::types::ModelicaParserFileSummary;
use crate::modelica_plugin::types::{ParsedDeclaration, ParsedImport};
use xiuxian_wendao_core::repo_intelligence::{ImportKind, RepoSymbolKind};

#[test]
fn modelica_parser_summary_file_semantic_fingerprint_changes_with_summary_semantics() {
    let base = ModelicaParserFileSummary {
        class_name: Some("Demo".to_string()),
        imports: vec![ParsedImport {
            name: "Modelica.SIunits".to_string(),
            alias: None,
            kind: ImportKind::Module,
            line_start: Some(1),
            attributes: BTreeMap::new(),
        }],
        declarations: vec![ParsedDeclaration {
            name: "Sample".to_string(),
            kind: RepoSymbolKind::Type,
            signature: "model Sample".to_string(),
            line_start: Some(2),
            line_end: Some(4),
            equations: vec![],
            attributes: BTreeMap::new(),
        }],
    };
    let same = base.clone();
    let changed = ModelicaParserFileSummary {
        class_name: Some("Demo".to_string()),
        imports: base.imports.clone(),
        declarations: vec![ParsedDeclaration {
            name: "ChangedSample".to_string(),
            kind: RepoSymbolKind::Type,
            signature: "model ChangedSample".to_string(),
            line_start: Some(2),
            line_end: Some(4),
            equations: vec![],
            attributes: BTreeMap::new(),
        }],
    };

    let base_fingerprint = modelica_parser_file_summary_semantic_fingerprint(&base);
    let same_fingerprint = modelica_parser_file_summary_semantic_fingerprint(&same);
    let changed_fingerprint = modelica_parser_file_summary_semantic_fingerprint(&changed);

    assert_eq!(base_fingerprint, same_fingerprint);
    assert_ne!(base_fingerprint, changed_fingerprint);
}

#[test]
fn modelica_file_semantic_fingerprint_changes_with_doc_surface_semantics() {
    let summary = ModelicaParserFileSummary {
        class_name: Some("DemoLib".to_string()),
        imports: Vec::new(),
        declarations: vec![ParsedDeclaration {
            name: "DemoLib".to_string(),
            kind: RepoSymbolKind::Type,
            signature: "package DemoLib".to_string(),
            line_start: Some(1),
            line_end: Some(3),
            equations: vec![],
            attributes: BTreeMap::new(),
        }],
    };

    let base_fingerprint = modelica_file_semantic_fingerprint(
        "package.mo",
        "within ;\npackage DemoLib\nend DemoLib;\n",
        &summary,
    );
    let changed_fingerprint = modelica_file_semantic_fingerprint(
        "package.mo",
        "within ;\npackage DemoLib\n  annotation(Documentation(info = \"doc\"));\nend DemoLib;\n",
        &summary,
    );

    assert_ne!(base_fingerprint, changed_fingerprint);
}

#[test]
fn modelica_file_semantic_fingerprint_changes_with_users_guide_section_docs() {
    let summary = ModelicaParserFileSummary {
        class_name: Some("ReleaseNotes".to_string()),
        imports: Vec::new(),
        declarations: vec![ParsedDeclaration {
            name: "ReleaseNotes".to_string(),
            kind: RepoSymbolKind::Type,
            signature: "package ReleaseNotes".to_string(),
            line_start: Some(1),
            line_end: Some(6),
            equations: vec![],
            attributes: BTreeMap::new(),
        }],
    };

    let base_fingerprint = modelica_file_semantic_fingerprint(
        "UsersGuide/ReleaseNotes.mo",
        "within DemoLib.UsersGuide;\npackage ReleaseNotes\nend ReleaseNotes;\n",
        &summary,
    );
    let changed_fingerprint = modelica_file_semantic_fingerprint(
        "UsersGuide/ReleaseNotes.mo",
        "within DemoLib.UsersGuide;\npackage ReleaseNotes\n  class Version_4_0\n    annotation(Documentation(info = \"doc\"));\n  end Version_4_0;\nend ReleaseNotes;\n",
        &summary,
    );

    assert_ne!(base_fingerprint, changed_fingerprint);
}
