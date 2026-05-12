//! Shared character comparison helpers for fuzzy search.

pub(crate) fn chars_equal_ignore_case(left: char, right: char) -> bool {
    left.to_lowercase().eq(right.to_lowercase())
}
