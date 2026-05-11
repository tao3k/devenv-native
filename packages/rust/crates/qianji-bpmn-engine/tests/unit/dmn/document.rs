use super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, parse_dmn_decision};

#[test]
fn dmn_parser_rejects_invalid_root_element() {
    let error = parse_dmn_decision(&fixture_source("invalid-root-element-decision-root.dmn"))
        .must_err("non-definitions DMN root should fail before decision parsing");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnRootElement {
            source_id: ("invalid-root-element-decision-root.dmn".to_string()).into(),
            element: "decision".to_string(),
        }
    );
}

#[test]
fn dmn_parser_rejects_missing_model_namespace() {
    let error = parse_dmn_decision(&fixture_source("invalid-missing-model-namespace.dmn"))
        .must_err("missing DMN model namespace should fail before decision parsing");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnModelNamespace {
            source_id: ("invalid-missing-model-namespace.dmn".to_string()).into(),
        }
    );
}

#[test]
fn dmn_parser_rejects_unsupported_model_namespace() {
    let error = parse_dmn_decision(&fixture_source(
        "invalid-unsupported-model-namespace-20211108.dmn",
    ))
    .must_err("unsupported DMN model namespace should fail before decision parsing");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnModelNamespace {
            source_id: ("invalid-unsupported-model-namespace-20211108.dmn".to_string()).into(),
            model_namespace_uri: "https://www.omg.org/spec/DMN/20211108/MODEL/".to_string(),
        }
    );
}

#[test]
fn dmn_parser_requires_definitions_namespace_attribute() {
    let error = parse_dmn_decision(&fixture_source(
        "invalid-missing-definitions-namespace-attribute.dmn",
    ))
    .must_err("definitions root should require the namespace attribute");

    assert_eq!(
        error,
        BpmnEngineError::MissingDmnAttribute {
            source_id: ("invalid-missing-definitions-namespace-attribute.dmn".to_string()).into(),
            element: "definitions".to_string(),
            attribute: "namespace".to_string(),
        }
    );
}

#[test]
fn dmn_parser_rejects_top_level_imports() {
    let error = parse_dmn_decision(&fixture_source("unsupported-top-level-import-20191111.dmn"))
        .must_err("top-level DMN imports should fail before decision parsing");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnImport {
            source_id: ("unsupported-top-level-import-20191111.dmn".to_string()).into(),
        }
    );
}
