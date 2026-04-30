use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};

use super::types::PdfOcrShardCacheResolution;
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_order::order_ocr_results_by_inputs;

impl PdfOcrShardCacheResolution {
    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn misses(
        &self,
    ) -> &[PdfOcrShardInput] {
        self.misses.as_slice()
    }

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn hit_count(
        &self,
    ) -> usize {
        self.hit_count
    }

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn merge(
        mut self,
        live_results: Vec<PdfOcrShardResult>,
    ) -> Result<Vec<PdfOcrShardResult>, String> {
        let ordered_live = order_ocr_results_by_inputs(self.misses.as_slice(), live_results)?;
        for (position, result) in self.miss_positions.into_iter().zip(ordered_live) {
            self.slots[position] = Some(result);
        }
        self.slots
            .into_iter()
            .enumerate()
            .map(|(position, result)| {
                result.ok_or_else(|| {
                    format!("OCR shard cache merge left input position {position} unresolved")
                })
            })
            .collect()
    }
}
