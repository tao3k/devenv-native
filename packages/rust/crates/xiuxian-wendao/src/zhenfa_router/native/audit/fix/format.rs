//! `zhenfa_router::native::audit::fix::format` owns Wendao audit fix format behavior.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::hash::BuildHasher;
use std::path::PathBuf;

use super::preview::FixPreview;

/// Generate a diff-style preview of fixes.
#[must_use]
pub fn format_fix_preview<S: BuildHasher>(
    previews: &HashMap<PathBuf, Vec<FixPreview>, S>,
) -> String {
    let mut output = String::new();

    macro_rules! append {
        ($($arg:tt)*) => {
            if write!(output, $($arg)*).is_err() {
                unreachable!("writing fix preview into String cannot fail");
            }
        };
    }

    for (path, file_previews) in previews {
        append!(
            "=== {} ({} fixes) ===\n",
            path.display(),
            file_previews.len()
        );

        for preview in file_previews {
            append!("{preview}\n\n");
        }
    }

    output
}
