//! `search::fuzzy::distance` owns Wendao search fuzzy distance behavior.

use super::buffers::{collect_chars, with_thread_local_buffers};
use super::case::chars_equal_ignore_case;

/// Compute the shared-prefix length in Unicode scalar values.
#[must_use]
pub fn shared_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| chars_equal_ignore_case(*left, *right))
        .count()
}

/// Check whether a candidate satisfies the prefix-length requirement.
#[must_use]
pub fn passes_prefix_requirement(query: &str, candidate: &str, prefix_length: usize) -> bool {
    if prefix_length == 0 {
        return true;
    }
    shared_prefix_len(query, candidate) >= prefix_length
}

/// Calculate classic Levenshtein distance without transposition support.
#[must_use]
pub fn levenshtein_distance(left: &str, right: &str) -> usize {
    edit_distance(left, right, false)
}

/// Calculate edit distance, optionally treating adjacent transpositions as one edit.
#[must_use]
pub fn edit_distance(left: &str, right: &str, transposition: bool) -> usize {
    with_thread_local_buffers(|buffers| {
        collect_chars(left, &mut buffers.left_chars);
        collect_chars(right, &mut buffers.right_chars);
        edit_distance_with_scratch(
            buffers.left_chars.as_slice(),
            buffers.right_chars.as_slice(),
            transposition,
            &mut buffers.distance_scratch,
        )
    })
}

pub(crate) fn edit_distance_with_scratch(
    left_chars: &[char],
    right_chars: &[char],
    transposition: bool,
    scratch: &mut Vec<usize>,
) -> usize {
    let left_len = left_chars.len();
    let right_len = right_chars.len();

    if left_len == 0 {
        return right_len;
    }
    if right_len == 0 {
        return left_len;
    }

    let row_len = right_len + 1;
    scratch.clear();
    scratch.resize(row_len.saturating_mul(3), 0);
    let (prev_prev_row, tail) = scratch.split_at_mut(row_len);
    let (prev_row, curr_row) = tail.split_at_mut(row_len);

    for (col_idx, cell) in prev_row.iter_mut().enumerate() {
        *cell = col_idx;
    }

    for left_idx in 1..=left_len {
        curr_row[0] = left_idx;
        for right_idx in 1..=right_len {
            let cost = usize::from(left_chars[left_idx - 1] != right_chars[right_idx - 1]);
            let deletion = prev_row[right_idx] + 1;
            let insertion = curr_row[right_idx - 1] + 1;
            let substitution = prev_row[right_idx - 1] + cost;
            let mut best = deletion.min(insertion).min(substitution);

            if transposition
                && left_idx > 1
                && right_idx > 1
                && left_chars[left_idx - 1] == right_chars[right_idx - 2]
                && left_chars[left_idx - 2] == right_chars[right_idx - 1]
            {
                best = best.min(prev_prev_row[right_idx - 2] + 1);
            }

            curr_row[right_idx] = best;
        }
        prev_prev_row.copy_from_slice(prev_row);
        prev_row.copy_from_slice(curr_row);
    }

    prev_row[right_len]
}
