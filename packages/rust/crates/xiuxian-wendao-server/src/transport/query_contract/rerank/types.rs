//! Rerank score types shared by Wendao Flight clients and servers.

/// One scored rerank candidate produced by the shared Rust-owned scorer.
#[cfg(feature = "transport")]
#[derive(Debug, Clone, PartialEq)]
pub struct RerankScoredCandidate {
    /// Stable candidate identifier carried through the rerank request.
    pub doc_id: String,
    /// Raw vector score from the rerank request.
    pub vector_score: f64,
    /// Semantic score derived from cosine similarity and normalized into `[0, 1]`.
    pub semantic_score: f64,
    /// Final blended rerank score.
    pub final_score: f64,
}

/// Shared runtime-owned rerank score weights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RerankScoreWeights {
    /// Weight applied to the inbound `vector_score`.
    pub vector_weight: f64,
    /// Weight applied to the derived `semantic_score`.
    pub semantic_weight: f64,
}

impl Default for RerankScoreWeights {
    fn default() -> Self {
        Self {
            vector_weight: 0.4,
            semantic_weight: 0.6,
        }
    }
}

impl RerankScoreWeights {
    /// Construct one validated rerank score-weight policy.
    ///
    /// # Errors
    ///
    /// Returns an error when either weight is non-finite, negative, or when
    /// both weights sum to zero.
    pub fn new(vector_weight: f64, semantic_weight: f64) -> Result<Self, String> {
        if !vector_weight.is_finite() {
            return Err("rerank vector_weight must be finite".to_string());
        }
        if !semantic_weight.is_finite() {
            return Err("rerank semantic_weight must be finite".to_string());
        }
        if vector_weight < 0.0 {
            return Err("rerank vector_weight must be greater than or equal to zero".to_string());
        }
        if semantic_weight < 0.0 {
            return Err("rerank semantic_weight must be greater than or equal to zero".to_string());
        }
        let total = vector_weight + semantic_weight;
        if total <= 0.0 {
            return Err("rerank score weights must sum to greater than zero".to_string());
        }
        Ok(Self {
            vector_weight,
            semantic_weight,
        })
    }

    /// Return the normalized score weights whose sum is exactly `1.0`.
    #[must_use]
    pub fn normalized(self) -> Self {
        let total = self.vector_weight + self.semantic_weight;
        Self {
            vector_weight: self.vector_weight / total,
            semantic_weight: self.semantic_weight / total,
        }
    }
}
