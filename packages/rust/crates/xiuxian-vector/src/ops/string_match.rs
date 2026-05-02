//! String matching kernels used by scalar column filters.

use crate::{LanceBooleanArray, LanceStringArray};

/// Compute a boolean mask for `contains` against one UTF-8 array and one scalar needle.
#[must_use]
pub fn string_contains_mask(values: &LanceStringArray, needle: &str) -> LanceBooleanArray {
    let matches = values
        .iter()
        .map(|candidate| candidate.map(|text| text.contains(needle)))
        .collect::<Vec<_>>();
    LanceBooleanArray::from(matches)
}
