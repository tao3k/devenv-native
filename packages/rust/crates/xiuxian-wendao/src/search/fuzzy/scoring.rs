//! `search::fuzzy::scoring` owns Wendao search fuzzy scoring behavior.

use crate::search::fuzzy::buffers::{
    collect_chars, collect_lowercase_chars, collect_lowercase_chars_and_shared_prefix,
    with_thread_local_buffers,
};
use crate::search::fuzzy::distance::edit_distance_with_scratch;
use crate::search::fuzzy::options::FuzzySearchOptions;
use crate::search::fuzzy::types::FuzzyScore;

/// Calculate normalized similarity score from edit distance.
#[must_use]
pub fn normalized_score(left: &str, right: &str, transposition: bool) -> f32 {
    with_thread_local_buffers(|buffers| {
        collect_chars(left, &mut buffers.left_chars);
        collect_chars(right, &mut buffers.right_chars);
        score_from_char_slices(
            buffers.left_chars.as_slice(),
            buffers.right_chars.as_slice(),
            transposition,
            &mut buffers.distance_scratch,
        )
    })
}

/// Score one candidate against one query using the shared options.
#[must_use]
pub fn score_candidate(
    query: &str,
    candidate: &str,
    options: FuzzySearchOptions,
) -> Option<FuzzyScore> {
    with_thread_local_buffers(|buffers| {
        collect_lowercase_chars(query, &mut buffers.left_chars);
        score_candidate_with_query_chars(
            query,
            buffers.left_chars.as_slice(),
            candidate,
            options,
            &mut buffers.right_chars,
            &mut buffers.distance_scratch,
        )
    })
}

pub(crate) fn score_candidate_with_query_chars(
    query: &str,
    query_chars: &[char],
    candidate: &str,
    options: FuzzySearchOptions,
    candidate_chars: &mut Vec<char>,
    scratch: &mut Vec<usize>,
) -> Option<FuzzyScore> {
    let shared_prefix =
        collect_lowercase_chars_and_shared_prefix(query, candidate, candidate_chars);
    if options.prefix_length > 0 && shared_prefix < options.prefix_length {
        return None;
    }

    score_candidate_from_char_slices(
        query_chars,
        candidate_chars.as_slice(),
        options.transposition,
        options.max_distance,
        scratch,
    )
}

fn normalized_score_from_distance(distance: usize, max_len: usize) -> f32 {
    1.0 - bounded_ratio(distance, max_len)
}

fn score_from_char_slices(
    left_chars: &[char],
    right_chars: &[char],
    transposition: bool,
    scratch: &mut Vec<usize>,
) -> f32 {
    let max_len = left_chars.len().max(right_chars.len());
    if max_len == 0 {
        return 1.0;
    }

    let distance = edit_distance_with_scratch(left_chars, right_chars, transposition, scratch);
    normalized_score_from_distance(distance, max_len)
}

fn score_candidate_from_char_slices(
    query_chars: &[char],
    candidate_chars: &[char],
    transposition: bool,
    max_distance: u8,
    scratch: &mut Vec<usize>,
) -> Option<FuzzyScore> {
    let max_len = query_chars.len().max(candidate_chars.len());
    let distance = edit_distance_with_scratch(query_chars, candidate_chars, transposition, scratch);
    if distance > usize::from(max_distance) {
        return None;
    }

    Some(FuzzyScore {
        score: if max_len == 0 {
            1.0
        } else {
            normalized_score_from_distance(distance, max_len)
        },
        distance,
    })
}

fn bounded_ratio(numerator: usize, denominator: usize) -> f32 {
    let numerator = bounded_usize_to_f32(numerator);
    let denominator = bounded_usize_to_f32(denominator.max(1));
    numerator / denominator
}

fn bounded_usize_to_f32(value: usize) -> f32 {
    u16::try_from(value).map_or(f32::from(u16::MAX), f32::from)
}
