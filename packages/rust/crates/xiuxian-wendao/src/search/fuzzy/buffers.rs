use std::{cell::RefCell, thread_local};

use super::case::chars_equal_ignore_case;

#[derive(Default)]
pub(crate) struct FuzzyThreadLocalBuffers {
    pub(crate) left_chars: Vec<char>,
    pub(crate) right_chars: Vec<char>,
    pub(crate) distance_scratch: Vec<usize>,
}

thread_local! {
    static FUZZY_THREAD_LOCAL_BUFFERS: RefCell<FuzzyThreadLocalBuffers> =
        RefCell::new(FuzzyThreadLocalBuffers::default());
}

pub(crate) fn with_thread_local_buffers<T>(
    operation: impl FnOnce(&mut FuzzyThreadLocalBuffers) -> T,
) -> T {
    FUZZY_THREAD_LOCAL_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        operation(&mut buffers)
    })
}

pub(crate) fn collect_chars(value: &str, target: &mut Vec<char>) {
    target.clear();
    target.extend(value.chars());
}

pub(crate) fn collect_lowercase_chars(value: &str, target: &mut Vec<char>) {
    target.clear();
    target.extend(value.chars().flat_map(char::to_lowercase));
}

pub(crate) fn collect_lowercase_chars_and_shared_prefix(
    query: &str,
    candidate: &str,
    target: &mut Vec<char>,
) -> usize {
    let mut query_chars = query.chars();
    let mut shared_prefix = 0;
    let mut prefix_matches = true;

    target.clear();
    for candidate_char in candidate.chars() {
        if prefix_matches {
            match query_chars.next() {
                Some(query_char) if chars_equal_ignore_case(query_char, candidate_char) => {
                    shared_prefix += 1;
                }
                _ => {
                    prefix_matches = false;
                }
            }
        }
        target.extend(candidate_char.to_lowercase());
    }

    shared_prefix
}
