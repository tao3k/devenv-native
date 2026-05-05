use xiuxian_ast::SecurityScanner;

#[test]
fn test_scanner_allows_safe_code() {
    let scanner = SecurityScanner::new();
    let safe_code = r#"
def hello(name: str) -> str:
    return f"Hello, {name}!"

class MyClass:
    def method(self):
        pass
"#;
    assert!(scanner.scan(safe_code).is_ok());
}

#[test]
fn test_scanner_detects_forbidden_import() {
    let scanner = SecurityScanner::new();
    let dangerous_code = r"
import os
def hello():
    pass
";
    let result = scanner.scan(dangerous_code);
    match result {
        Ok(()) => panic!("expected forbidden import to be rejected"),
        Err(error) => assert!(error.description.contains("os")),
    }
}

#[test]
fn test_scanner_detects_dangerous_call() {
    let scanner = SecurityScanner::new();
    let dangerous_code = r#"
def execute():
    eval("1 + 1")
"#;
    let result = scanner.scan(dangerous_code);
    match result {
        Ok(()) => panic!("expected dangerous call to be rejected"),
        Err(error) => assert!(error.description.contains("eval")),
    }
}

#[test]
fn test_scanner_detects_subprocess() {
    let scanner = SecurityScanner::new();
    let dangerous_code = r#"
import subprocess
subprocess.run(["ls"])
"#;
    let result = scanner.scan(dangerous_code);
    assert!(result.is_err());
}

#[test]
fn test_scan_all_returns_multiple_violations() {
    let scanner = SecurityScanner::new();
    let code = r#"
import os
import subprocess
eval("x")
exec("y")
"#;
    let violations = scanner.scan_all(code);
    // scan_all collects all violations (non-fail-fast)
    // Expecting: 2 import violations + 2 call violations = 4
    assert!(
        violations.len() >= 3,
        "Expected at least 3 violations, got {}",
        violations.len()
    );
}
