use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnEngineError, parse_dmn_decision};

#[test]
fn dmn_parser_rejects_mixed_duration_family_ranges() {
    let error = parse_dmn_decision(&fixture_source("invalid-mixed-duration-family-range.dmn"))
        .must_err("mixed duration-family ranges should stay explicit");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: ("invalid-mixed-duration-family-range.dmn".to_string()).into(),
            expression: "duration(\"P6M\")<= ?<duration(\"P30D\")".to_string(),
        }
    );
}
