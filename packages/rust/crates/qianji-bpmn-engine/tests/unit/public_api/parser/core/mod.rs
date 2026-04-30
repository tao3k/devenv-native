use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnPackage, BpmnParseOptions, parse_bpmn_package};

mod gateway;
mod linear;
mod repeat;
mod task;
mod task_io;

fn parse_fixture_package(name: &str) -> BpmnPackage {
    parse_bpmn_package(&[fixture_source(name)], &BpmnParseOptions::default())
        .must("bounded BPMN subset should parse")
}

fn parse_fixture_error(name: &str, context: &str) -> BpmnEngineError {
    parse_bpmn_package(&[fixture_source(name)], &BpmnParseOptions::default()).must_err(context)
}
