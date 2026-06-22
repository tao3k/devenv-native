#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecurityViolation {
    pub(crate) rule_id: &'static str,
    pub(crate) description: String,
    pub(crate) line: usize,
    pub(crate) snippet: String,
}

const FORBIDDEN_IMPORTS: &[&str] = &[
    "os",
    "subprocess",
    "socket",
    "ctypes",
    "threading",
    "multiprocessing",
];
const FORBIDDEN_CALLS: &[&str] = &["eval", "exec", "execfile", "compile", "open", "__import__"];
const SUSPICIOUS_CALLS: &[(&str, &str)] = &[
    ("getattr", "Dynamic attribute access via getattr()"),
    ("setattr", "Dynamic attribute setting via setattr()"),
    ("globals", "Access to globals()"),
    ("locals", "Access to locals()"),
];

pub(crate) fn scan_security_violations(code: &str) -> Vec<SecurityViolation> {
    code.lines()
        .enumerate()
        .flat_map(|(index, line)| scan_line(index + 1, line))
        .collect()
}

fn scan_line(line_number: usize, line: &str) -> Vec<SecurityViolation> {
    let trimmed = line.trim_start();
    let import_violations = FORBIDDEN_IMPORTS
        .iter()
        .filter_map(|module| forbidden_import_violation(line_number, line, trimmed, module));
    let call_violations = FORBIDDEN_CALLS
        .iter()
        .filter(|call| contains_call(trimmed, call))
        .map(|call| SecurityViolation {
            rule_id: "SEC-CALL-001",
            description: format!("Dangerous call: '{call}()' is not allowed"),
            line: line_number,
            snippet: snippet(line),
        });
    let suspicious_violations = SUSPICIOUS_CALLS
        .iter()
        .filter(|(call, _)| contains_call(trimmed, call))
        .map(|(_, description)| SecurityViolation {
            rule_id: "SEC-PATTERN-001",
            description: (*description).to_owned(),
            line: line_number,
            snippet: snippet(line),
        });

    import_violations
        .chain(call_violations)
        .chain(suspicious_violations)
        .collect()
}

fn forbidden_import_violation(
    line_number: usize,
    line: &str,
    trimmed: &str,
    module: &str,
) -> Option<SecurityViolation> {
    if let Some(rest) = trimmed.strip_prefix("import ")
        && imported_module_matches(rest, module)
    {
        return Some(SecurityViolation {
            rule_id: "SEC-IMPORT-001",
            description: format!("Forbidden import: '{module}' is not allowed in skills"),
            line: line_number,
            snippet: snippet(line),
        });
    }
    if let Some(rest) = trimmed.strip_prefix("from ")
        && imported_module_matches(rest, module)
    {
        return Some(SecurityViolation {
            rule_id: "SEC-IMPORT-001",
            description: format!("Forbidden import from: '{module}' is not allowed"),
            line: line_number,
            snippet: snippet(line),
        });
    }
    None
}

fn imported_module_matches(rest: &str, module: &str) -> bool {
    rest == module
        || rest
            .strip_prefix(module)
            .is_some_and(|suffix| suffix.starts_with([' ', '\t', ',', '.', ';']))
}

fn contains_call(line: &str, call: &str) -> bool {
    let mut remainder = line;
    while let Some(index) = remainder.find(call) {
        let before = if index == 0 {
            None
        } else {
            remainder[..index].chars().next_back()
        };
        let after = remainder[index + call.len()..].trim_start();
        if before.is_none_or(|character| !is_identifier_character(character))
            && after.starts_with('(')
        {
            return true;
        }
        remainder = &remainder[index + call.len()..];
    }
    false
}

fn is_identifier_character(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn snippet(line: &str) -> String {
    line.trim().chars().take(80).collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/executors/security_scan/scanner.rs"]
mod tests;
