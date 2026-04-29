use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnPackage, BpmnParseOptions, parse_bpmn_package};

mod boundary;
mod errors;
mod terminate;
mod waits;

fn parse_fixture_package(name: &str, context: &str) -> BpmnPackage {
    parse_bpmn_package(&[fixture_source(name)], &BpmnParseOptions::default()).must(context)
}

fn parse_fixture_error(name: &str, context: &str) -> BpmnEngineError {
    parse_bpmn_package(&[fixture_source(name)], &BpmnParseOptions::default()).must_err(context)
}
