//! Shared `SearchStrategyFlow` candidate contracts.

use serde::Serialize;

pub(crate) const MAX_CANDIDATES: usize = 12;
pub(crate) const MARKDOWN_HEADING_CANDIDATE_SOURCE: &str = "rust-markdown-headings";
pub(crate) const CODE_INTELLIGENCE_CANDIDATE_SOURCE: &str = "rust-code-intelligence-inventory";
pub(crate) const WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE: &str = "wendao-gateway-retrieval";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchStrategyFlowCandidateInput {
    pub(crate) relative_path: String,
    pub(crate) heading_anchor: String,
    pub(crate) title: String,
    pub(crate) line_start: usize,
    pub(crate) line_end: usize,
    pub(crate) context_cost: usize,
    pub(crate) evidence_coverage: f64,
    pub(crate) graph_score: f64,
    pub(crate) authority_score: f64,
    pub(crate) structural_score: f64,
    pub(crate) uncertainty: f64,
    pub(crate) blocked: bool,
    pub(crate) edge_kinds: Vec<String>,
}

/// Candidate batch passed from Rust discovery into `SearchStrategyFlow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStrategyFlowCandidateInputBatch {
    pub(crate) source: &'static str,
    pub(crate) row_count: usize,
    pub(crate) candidate_input_arrow_ipc_stream: Vec<u8>,
    pub(crate) discovery_receipt_json: String,
}

impl SearchStrategyFlowCandidateInputBatch {
    pub(crate) fn candidate_input_arrow_ipc_byte_len(&self) -> usize {
        self.candidate_input_arrow_ipc_stream.len()
    }

    #[cfg(test)]
    pub(crate) fn candidate_input_arrow_snapshot(&self) -> String {
        use std::io::Cursor;

        use arrow::array::{Array, BooleanArray, Float64Array, Int64Array, StringArray};
        use arrow_ipc::reader::StreamReader;

        if self.candidate_input_arrow_ipc_stream.is_empty() {
            return String::new();
        }
        let reader = StreamReader::try_new(
            Cursor::new(self.candidate_input_arrow_ipc_stream.as_slice()),
            None,
        )
        .expect("candidate Arrow IPC stream should decode");
        let mut rows = Vec::new();
        for batch in reader {
            let batch = batch.expect("candidate Arrow IPC batch should decode");
            let candidate_id = string_column(&batch, "candidate_id");
            let relative_path = string_column(&batch, "relative_path");
            let heading_anchor = string_column(&batch, "heading_anchor");
            let title = string_column(&batch, "title");
            let candidate_kind = string_column(&batch, "candidate_kind");
            let edge_kinds = string_column(&batch, "edge_kinds");
            let line_start = int_column(&batch, "line_start");
            let line_end = int_column(&batch, "line_end");
            let context_cost = int_column(&batch, "context_cost");
            let evidence_coverage = float_column(&batch, "evidence_coverage");
            let graph_score = float_column(&batch, "graph_score");
            let authority_score = float_column(&batch, "authority_score");
            let structural_score = float_column(&batch, "structural_score");
            let uncertainty = float_column(&batch, "uncertainty");
            let blocked = bool_column(&batch, "blocked");
            for row_index in 0..batch.num_rows() {
                rows.push(format!(
                    "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    snapshot_value(relative_path, row_index),
                    snapshot_value(heading_anchor, row_index),
                    snapshot_value(title, row_index),
                    snapshot_value(candidate_id, row_index),
                    snapshot_value(candidate_kind, row_index),
                    snapshot_value(edge_kinds, row_index),
                    line_start.value(row_index),
                    line_end.value(row_index),
                    context_cost.value(row_index),
                    evidence_coverage.value(row_index),
                    graph_score.value(row_index),
                    authority_score.value(row_index),
                    structural_score.value(row_index),
                    uncertainty.value(row_index),
                    blocked.value(row_index),
                ));
            }
        }

        fn string_column<'a>(
            batch: &'a arrow::record_batch::RecordBatch,
            name: &str,
        ) -> &'a StringArray {
            batch
                .column_by_name(name)
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should exist"))
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should be Utf8"))
        }

        fn int_column<'a>(
            batch: &'a arrow::record_batch::RecordBatch,
            name: &str,
        ) -> &'a Int64Array {
            batch
                .column_by_name(name)
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should exist"))
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should be Int64"))
        }

        fn float_column<'a>(
            batch: &'a arrow::record_batch::RecordBatch,
            name: &str,
        ) -> &'a Float64Array {
            batch
                .column_by_name(name)
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should exist"))
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should be Float64"))
        }

        fn bool_column<'a>(
            batch: &'a arrow::record_batch::RecordBatch,
            name: &str,
        ) -> &'a BooleanArray {
            batch
                .column_by_name(name)
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should exist"))
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap_or_else(|| panic!("candidate Arrow column `{name}` should be Boolean"))
        }

        fn string_value(array: &StringArray, row_index: usize) -> &str {
            assert!(!array.is_null(row_index));
            array.value(row_index)
        }

        fn snapshot_value(array: &StringArray, row_index: usize) -> String {
            string_value(array, row_index)
                .replace('\\', "\\\\")
                .replace('|', "\\|")
                .replace('\t', "\\t")
                .replace('\r', "\\r")
                .replace('\n', "\\n")
        }

        rows.join("\n")
    }
}

pub(crate) struct SearchStrategyFlowRepoSearchHit<'a> {
    pub(crate) relative_path: &'a str,
    pub(crate) title: Option<&'a str>,
    pub(crate) best_section: Option<&'a str>,
    pub(crate) line_start: Option<usize>,
    pub(crate) line_end: Option<usize>,
    pub(crate) score: Option<f64>,
}
