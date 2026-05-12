//! `link_graph::saliency::signal` owns Wendao link graph saliency signal behavior.

use super::LinkGraphSaliencyState;

/// Map a saliency state into a normalized learning signal.
#[must_use]
pub fn learned_saliency_signal_from_state(state: &LinkGraphSaliencyState) -> f64 {
    state.current_saliency
}
