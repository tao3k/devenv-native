//! Typed runtime configuration records loaded from Wendao config sources.

mod agentic;
mod cache;
mod coactivation;
mod index;
mod related;

pub use agentic::LinkGraphAgenticRuntimeConfig;
pub use cache::LinkGraphCacheRuntimeConfig;
pub use coactivation::LinkGraphCoactivationRuntimeConfig;
pub use index::LinkGraphIndexRuntimeConfig;
pub use related::LinkGraphRelatedRuntimeConfig;
