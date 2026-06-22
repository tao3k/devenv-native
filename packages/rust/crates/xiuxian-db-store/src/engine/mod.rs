//! Engine-facing Arrow/DataFusion storage facade and non-vector fallback exports.

mod context;
mod parquet;
mod query_support;
mod stub;

pub use context::{SearchEngineContext, SearchEnginePartitionColumn};
pub use parquet::{write_engine_batches_to_parquet_file, write_lance_batches_to_parquet_file};
pub use query_support::{
    RETRIEVAL_BEST_SECTION_COLUMN, RETRIEVAL_DOC_TYPE_COLUMN, RETRIEVAL_ID_COLUMN,
    RETRIEVAL_LANGUAGE_COLUMN, RETRIEVAL_LINE_COLUMN, RETRIEVAL_MATCH_REASON_COLUMN,
    RETRIEVAL_PATH_COLUMN, RETRIEVAL_REPO_COLUMN, RETRIEVAL_SCORE_COLUMN, RETRIEVAL_SNIPPET_COLUMN,
    RETRIEVAL_SOURCE_COLUMN, RETRIEVAL_TITLE_COLUMN, RetrievalDocType, RetrievalRow,
    payload_fetch_record_batch, retrieval_result_columns, retrieval_result_schema,
    retrieval_rows_from_record_batch, retrieval_rows_to_record_batch,
};
pub use stub::{ColumnarScanOptions, TableInfo, VectorStore};
