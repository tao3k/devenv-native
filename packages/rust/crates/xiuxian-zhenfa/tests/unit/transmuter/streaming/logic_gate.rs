use crate::transmuter::streaming::logic_gate::{LogicGate, LogicGateError, XsdConstraintMap};

#[test]
fn logic_gate_accepts_valid_qianji_plan() {
    let mut gate = LogicGate::new();
    let xml = r#"<?xml version="1.0"?>
<qianji-audit-plan version="1.0">
  <summary>
    <intent>Test</intent>
    <total-steps>1</total-steps>
  </summary>
</qianji-audit-plan>"#;

    let result = gate.hot_validate(xml);
    assert!(result.is_ok());
}

#[test]
fn logic_gate_rejects_forbidden_tag() {
    let mut gate = LogicGate::new();
    let xml = r"<invalid-root></invalid-root>";

    let result = gate.hot_validate(xml);
    assert!(matches!(result, Err(LogicGateError::ForbiddenTag { .. })));
}

#[test]
fn logic_gate_rejects_invalid_action() {
    let mut gate = LogicGate::new();
    let xml = r#"<qianji-audit-plan version="1.0">
  <summary><intent>Test</intent><total-steps>1</total-steps></summary>
  <implementation-steps>
    <step number="1">
      <file-target path="test.rs" action="ship"/>
    </step>
  </implementation-steps>
</qianji-audit-plan>"#;

    let result = gate.hot_validate(xml);
    assert!(matches!(
        result,
        Err(LogicGateError::InvalidAttribute { .. })
    ));
}

#[test]
fn logic_gate_rejects_non_sequential_steps() {
    let mut gate = LogicGate::new();
    let xml = r#"<qianji-audit-plan version="1.0">
  <summary><intent>Test</intent><total-steps>2</total-steps></summary>
  <implementation-steps>
    <step number="2"><title>Skip</title></step>
  </implementation-steps>
</qianji-audit-plan>"#;

    let result = gate.hot_validate(xml);
    assert!(matches!(
        result,
        Err(LogicGateError::NonSequentialStep { .. })
    ));
}

#[test]
fn xsd_constraint_map_allows_known_children() {
    let map = XsdConstraintMap::qianji_audit_plan();

    assert!(map.is_allowed_child("qianji-audit-plan", "summary"));
    assert!(map.is_allowed_child("implementation-steps", "step"));
    assert!(!map.is_allowed_child("summary", "step"));
}

#[test]
fn logic_gate_uses_static_constraint_map() {
    let gate1 = LogicGate::new();
    let gate2 = LogicGate::new();

    assert!(std::ptr::eq(gate1.constraints, gate2.constraints));
}

#[test]
fn static_constraint_map_lazy_initialization() {
    let gate1 = LogicGate::new();
    let gate2 = LogicGate::new();

    assert!(std::ptr::eq(gate1.constraints, gate2.constraints));
}

#[test]
fn logic_gate_test_helpers_report_parser_state() {
    let mut gate = LogicGate::with_constraints(XsdConstraintMap::qianji_audit_plan());
    assert!(gate.is_complete());
    assert_eq!(gate.depth(), 0);
    assert_eq!(gate.current_parent(), None);

    let xml = r"<qianji-audit-plan><summary>";
    let result = gate.hot_validate(xml);
    assert!(result.is_ok());
    assert!(!gate.is_complete());
    assert_eq!(gate.depth(), 2);
    assert_eq!(gate.current_parent(), Some("summary"));
}
