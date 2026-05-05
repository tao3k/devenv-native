use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, parse_dmn_decision};

#[test]
fn dmn_parser_exact_one_wrapper_rejects_multiple_decisions() {
    let error = parse_dmn_decision(&fixture_source("multiple-decisions.dmn"))
        .must_err("exact-one wrapper should stay explicit for multi-decision sources");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnDecisionCount {
            source_id: "multiple-decisions.dmn".to_string(),
            count: 2,
        }
    );
}

#[test]
fn dmn_parser_rejects_unsupported_unary_tests() {
    let error = parse_dmn_decision(&fixture_source("invalid-unsupported-unary-test.dmn"))
        .must_err("unsupported unary tests should stay explicit");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: "invalid-unsupported-unary-test.dmn".to_string(),
            expression: "duration(\"P1.5Y\")".to_string(),
        }
    );
}

#[test]
fn dmn_parser_rejects_unsupported_direct_list_children() {
    let error = parse_dmn_decision(&fixture_source("invalid-list-unsupported-child.dmn"))
        .must_err("bounded direct list parser should reject non-literal children");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_list_unsupported_child",
        }
    );
}

#[test]
fn dmn_parser_rejects_unsupported_direct_context_children() {
    let error = parse_dmn_decision(&fixture_source("invalid-context-unsupported-child.dmn"))
        .must_err("bounded direct context parser should reject non-entry children");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_context_unsupported_child",
        }
    );
}

#[test]
fn dmn_parser_rejects_unsupported_direct_relation_children() {
    let error = parse_dmn_decision(&fixture_source("invalid-relation-unsupported-child.dmn"))
        .must_err("bounded direct relation parser should reject non-cell children");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedOperation {
            operation: "parse_dmn_relation_unsupported_child",
        }
    );
}
