//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::search::ranking::RetainedWindow;

const MIN_RETAINED_LOCAL_SYMBOLS: usize = 64;
const RETAINED_LOCAL_SYMBOL_MULTIPLIER: usize = 4;
const MIN_RETAINED_AUTOCOMPLETE_SUGGESTIONS: usize = 16;
const RETAINED_AUTOCOMPLETE_MULTIPLIER: usize = 2;

pub(crate) fn retained_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(
        limit,
        RETAINED_LOCAL_SYMBOL_MULTIPLIER,
        MIN_RETAINED_LOCAL_SYMBOLS,
    )
}

pub(crate) fn suggestion_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(
        limit,
        RETAINED_AUTOCOMPLETE_MULTIPLIER,
        MIN_RETAINED_AUTOCOMPLETE_SUGGESTIONS,
    )
}
