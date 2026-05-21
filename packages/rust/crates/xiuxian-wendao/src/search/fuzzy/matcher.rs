//! `search::fuzzy::matcher` owns Wendao search fuzzy matcher behavior.

use std::cmp::Ordering;
use std::convert::Infallible;

use crate::search::fuzzy::buffers::{collect_lowercase_chars, with_thread_local_buffers};
use crate::search::fuzzy::options::FuzzySearchOptions;
use crate::search::fuzzy::scoring::score_candidate_with_query_chars;
use crate::search::fuzzy::types::{FuzzyMatch, FuzzyMatcher};

/// Generic lexical matcher over an in-memory candidate slice.
pub struct LexicalMatcher<'a, T, F> {
    candidates: &'a [T],
    extract: F,
    options: FuzzySearchOptions,
}

impl<'a, T, F> LexicalMatcher<'a, T, F> {
    /// Create a lexical matcher.
    #[must_use]
    pub fn new(candidates: &'a [T], extract: F, options: FuzzySearchOptions) -> Self {
        Self {
            candidates,
            extract,
            options,
        }
    }

    /// Access the matcher options.
    #[must_use]
    pub const fn options(&self) -> FuzzySearchOptions {
        self.options
    }
}

impl<T, F> FuzzyMatcher<T> for LexicalMatcher<'_, T, F>
where
    T: Clone,
    F: for<'b> Fn(&'b T) -> &'b str,
{
    type Error = Infallible;

    fn search(&self, query: &str, limit: usize) -> Result<Vec<FuzzyMatch<T>>, Self::Error> {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let mut matches = with_thread_local_buffers(|buffers| {
            collect_lowercase_chars(query, &mut buffers.left_chars);

            let mut matches = Vec::new();
            for candidate in self.candidates {
                let matched_text = (self.extract)(candidate);
                if let Some(score) = score_candidate_with_query_chars(
                    query,
                    buffers.left_chars.as_slice(),
                    matched_text,
                    self.options,
                    &mut buffers.right_chars,
                    &mut buffers.distance_scratch,
                ) {
                    matches.push(FuzzyMatch {
                        item: candidate.clone(),
                        matched_text: matched_text.to_string(),
                        score: score.score,
                        distance: score.distance,
                    });
                }
            }

            matches
        });

        matches.sort_by(compare_fuzzy_matches);
        matches.truncate(limit);
        Ok(matches)
    }
}

fn compare_fuzzy_matches<T>(left: &FuzzyMatch<T>, right: &FuzzyMatch<T>) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.distance.cmp(&right.distance))
        .then_with(|| left.matched_text.len().cmp(&right.matched_text.len()))
        .then_with(|| left.matched_text.cmp(&right.matched_text))
}
