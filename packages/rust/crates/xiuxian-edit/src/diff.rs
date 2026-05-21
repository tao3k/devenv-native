//! Diff generation utilities.
//!
//! Provides unified diff output for structural edits using the `similar` crate.

use similar::{ChangeTag, TextDiff};

/// Generate a unified diff between two strings.
///
/// Uses the `similar` crate for line-by-line diffing with context.
///
/// # Arguments
/// * `original` - The original content
/// * `modified` - The modified content
///
/// # Returns
/// A string containing the unified diff with `+`, `-`, and ` ` prefixes.
#[must_use]
pub fn generate_unified_diff(original: &str, modified: &str) -> String {
    render_unified_diff(&TextDiff::from_lines(original, modified))
}

fn render_unified_diff(diff: &TextDiff<'_, '_, '_, str>) -> String {
    let mut output = String::new();

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        append_group_separator(&mut output, idx);
        for op in group {
            for change in diff.iter_changes(op) {
                append_change(&mut output, &change);
            }
        }
    }

    output
}

fn append_group_separator(output: &mut String, group_index: usize) {
    if group_index > 0 {
        output.push_str("...\n");
    }
}

fn append_change(output: &mut String, change: &similar::Change<&str>) {
    output.push_str(change_sign(change.tag()));
    output.push_str(change.value());
    if change.missing_newline() {
        output.push('\n');
    }
}

fn change_sign(tag: ChangeTag) -> &'static str {
    match tag {
        ChangeTag::Delete => "-",
        ChangeTag::Insert => "+",
        ChangeTag::Equal => " ",
    }
}

#[cfg(test)]
#[path = "../tests/unit/diff.rs"]
mod tests;
