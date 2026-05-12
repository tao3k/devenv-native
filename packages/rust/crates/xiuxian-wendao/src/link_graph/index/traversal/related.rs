use crate::link_graph::index::{
    LinkGraphIndex, LinkGraphNeighbor, LinkGraphRelatedPprDiagnostics, LinkGraphRelatedPprOptions,
};

impl LinkGraphIndex {
    /// Find related notes from a seed note stem or id.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn related(
        &self,
        stem_or_id: &str,
        max_distance: usize,
        limit: usize,
    ) -> Vec<LinkGraphNeighbor> {
        let (rows, _) = self.related_with_diagnostics(stem_or_id, max_distance, limit, None);
        rows
    }

    /// Find related notes and return PPR diagnostics.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn related_with_diagnostics(
        &self,
        stem_or_id: &str,
        max_distance: usize,
        limit: usize,
        ppr: Option<&LinkGraphRelatedPprOptions>,
    ) -> (
        Vec<LinkGraphNeighbor>,
        Option<LinkGraphRelatedPprDiagnostics>,
    ) {
        let seeds = vec![stem_or_id.to_string()];
        self.related_from_seeds_with_diagnostics(&seeds, max_distance, limit, ppr)
    }

    /// Find related notes from explicit seed notes.
    #[must_use]
    pub fn related_from_seeds(
        &self,
        seeds: &[String],
        max_distance: usize,
        limit: usize,
    ) -> Vec<LinkGraphNeighbor> {
        let (rows, _) = self.related_from_seeds_with_diagnostics(seeds, max_distance, limit, None);
        rows
    }
}
