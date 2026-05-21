//! `link_graph::saliency::types` owns Wendao link graph saliency types behavior.

use serde::{Deserialize, Serialize};

/// Namespace-scoped schema version for `LinkGraph` saliency persistence.
pub const LINK_GRAPH_SALIENCY_SCHEMA_VERSION: &str = "xiuxian_wendao.link_graph.saliency.v1";

/// Default frontmatter/in-memory saliency base when not explicitly configured.
pub const DEFAULT_SALIENCY_BASE: f64 = 5.0;
/// Default frontmatter/in-memory saliency decay rate when not explicitly configured.
pub const DEFAULT_DECAY_RATE: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Runtime policy that clamps and scales saliency updates.
pub struct LinkGraphSaliencyPolicy {
    /// Activation boost coefficient.
    pub alpha: f64,
    /// Lower clamp bound for saliency.
    pub minimum: f64,
    /// Upper clamp bound for saliency.
    pub maximum: f64,
}

impl Default for LinkGraphSaliencyPolicy {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            minimum: 1.0,
            maximum: 10.0,
        }
    }
}

impl LinkGraphSaliencyPolicy {
    /// Normalize policy values into a stable numeric range.
    #[must_use]
    pub fn normalized(self) -> Self {
        let alpha = if self.alpha.is_finite() {
            self.alpha.max(0.0)
        } else {
            Self::default().alpha
        };
        let minimum = if self.minimum.is_finite() {
            self.minimum
        } else {
            Self::default().minimum
        };
        let maximum_raw = if self.maximum.is_finite() {
            self.maximum
        } else {
            Self::default().maximum
        };
        let maximum = if maximum_raw >= minimum {
            maximum_raw
        } else {
            minimum
        };
        Self {
            alpha,
            minimum,
            maximum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Persisted saliency snapshot for a single `LinkGraph` node.
pub struct LinkGraphSaliencyState {
    /// Persistence schema version.
    pub schema: String,
    /// Canonical graph node id.
    pub node_id: String,
    /// Baseline saliency used for next settlement.
    pub saliency_base: f64,
    /// Exponential decay rate.
    pub decay_rate: f64,
    /// Historical touch count (observability only). The online score update uses
    /// per-touch delta and settles new saliency as the next baseline.
    pub activation_count: u64,
    /// Last access timestamp (unix seconds).
    pub last_accessed_unix: i64,
    /// Current settled saliency value.
    pub current_saliency: f64,
    /// Last update timestamp (unix seconds as float).
    pub updated_at_unix: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Request payload used to apply one saliency touch/update operation.
pub struct LinkGraphSaliencyTouchRequest {
    /// Target graph node id.
    pub node_id: String,
    /// Activation delta applied by this touch.
    #[serde(default)]
    pub activation_delta: u64,
    /// Optional baseline override.
    #[serde(default)]
    pub saliency_base: Option<f64>,
    /// Optional decay-rate override.
    #[serde(default)]
    pub decay_rate: Option<f64>,
    /// Optional policy alpha override.
    #[serde(default)]
    pub alpha: Option<f64>,
    /// Optional policy minimum override.
    #[serde(default)]
    pub minimum_saliency: Option<f64>,
    /// Optional policy maximum override.
    #[serde(default)]
    pub maximum_saliency: Option<f64>,
    /// Optional timestamp override for deterministic tests.
    #[serde(default)]
    pub now_unix: Option<i64>,
}

impl Default for LinkGraphSaliencyTouchRequest {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            activation_delta: 1,
            saliency_base: None,
            decay_rate: None,
            alpha: None,
            minimum_saliency: None,
            maximum_saliency: None,
            now_unix: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
/// Request payload used to settle all persisted saliency states forward in time.
pub struct LinkGraphSaliencyDecaySweepRequest {
    /// Optional timestamp override for deterministic tests and manual backfills.
    #[serde(default)]
    pub now_unix: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Summary of one global saliency decay sweep run.
pub struct LinkGraphSaliencyDecaySweepResult {
    /// Sweep settlement timestamp (unix seconds).
    pub now_unix: i64,
    /// Number of matching saliency keys scanned.
    pub scanned_keys: usize,
    /// Number of valid saliency states updated by the sweep.
    pub updated_states: usize,
    /// Number of invalid saliency payloads removed during the sweep.
    pub deleted_states: usize,
}
