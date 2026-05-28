use super::scan_security_violations;

#[test]
fn flags_forbidden_imports_and_calls() {
    let violations = scan_security_violations("import os\nsafe()\nvalue = eval('1 + 1')");

    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].rule_id, "SEC-IMPORT-001");
    assert_eq!(violations[0].line, 1);
    assert_eq!(violations[1].rule_id, "SEC-CALL-001");
    assert_eq!(violations[1].line, 3);
}

#[test]
fn ignores_identifier_substrings() {
    let violations = scan_security_violations("evaluate()\nfrom oscar import name");

    assert!(violations.is_empty());
}
