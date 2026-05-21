//! `link_graph::saliency::store` owns Wendao link graph saliency store behavior.

mod common;
mod decay;
mod read;
#[path = "write/mod.rs"]
mod write;

pub use decay::{valkey_saliency_decay_all, valkey_saliency_decay_all_with_valkey};
pub use read::{
    valkey_saliency_get, valkey_saliency_get_many, valkey_saliency_get_many_with_valkey,
    valkey_saliency_get_with_valkey,
};
pub use write::{valkey_saliency_del, valkey_saliency_touch, valkey_saliency_touch_with_valkey};
