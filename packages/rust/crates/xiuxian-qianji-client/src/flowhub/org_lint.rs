use std::path::Path;

use orgize::Org;

pub(crate) fn validate_org_syntax(
    path: &Path,
    source: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    let _document = Org::parse(source).document();
    let mut passed = true;
    if source.trim().is_empty() {
        diagnostics.push(format!("Org source `{}` is empty", path.display()));
        passed = false;
    }
    if !source
        .lines()
        .any(|line| line.trim_start().starts_with('*'))
    {
        diagnostics.push(format!(
            "Org source `{}` has no heading for Flowhub review",
            path.display()
        ));
        passed = false;
    }
    passed
}
