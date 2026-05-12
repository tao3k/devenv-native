//! `link_graph::saliency` owns Wendao link graph saliency behavior.

mod calc;
mod keys;
mod signal;
#[path = "store/mod.rs"]
mod store;
mod touch;
mod types;

pub use calc::compute_link_graph_saliency;
pub(crate) use keys::{edge_in_key, edge_out_key, saliency_key};
pub use signal::learned_saliency_signal_from_state;
pub use store::{
    valkey_saliency_decay_all, valkey_saliency_decay_all_with_valkey, valkey_saliency_del,
    valkey_saliency_get, valkey_saliency_get_many, valkey_saliency_get_many_with_valkey,
    valkey_saliency_get_with_valkey, valkey_saliency_touch, valkey_saliency_touch_with_valkey,
};
pub use touch::{
    SearchHitCoactivationLink, touch_search_hits_async, touch_search_hits_async_with_valkey,
    touch_search_hits_with_coactivation_async,
    touch_search_hits_with_coactivation_async_with_valkey,
};
pub use types::{
    DEFAULT_DECAY_RATE, DEFAULT_SALIENCY_BASE, LINK_GRAPH_SALIENCY_SCHEMA_VERSION,
    LinkGraphSaliencyDecaySweepRequest, LinkGraphSaliencyDecaySweepResult, LinkGraphSaliencyPolicy,
    LinkGraphSaliencyState, LinkGraphSaliencyTouchRequest,
};
