//! `link_graph::runtime_config::models::retrieval` owns Wendao runtime config models retrieval behavior.

mod policy;
mod semantic_ignition;

pub use policy::LinkGraphRetrievalPolicyRuntimeConfig;
pub use semantic_ignition::LinkGraphSemanticIgnitionBackend;
#[cfg(feature = "vector-store")]
pub use semantic_ignition::LinkGraphSemanticIgnitionRuntimeConfig;
