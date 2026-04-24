//! Administrative and maintenance operations for `VectorStore`.

mod cache;
pub mod column_read;
mod columnar;
mod maintenance;
mod migration;
mod observability;
mod partitioning;
mod scalar;
mod string_match;
mod types;
mod vector_index;

pub use crate::ScalarIndexType;
pub use cache::{DatasetCache, DatasetCacheConfig};
pub use column_read::{get_intents_at, get_routing_keywords_at, get_utf8_at};
pub use columnar::ColumnarScanOptions;
pub use migration::{
    MigrateResult, MigrationItem, XIUXIAN_SCHEMA_VERSION, schema_version_from_schema,
};
pub use string_match::string_contains_mask;
pub use types::{
    CompactionStats, DocumentRow, FragmentInfo, IndexBuildProgress, IndexCacheStats, IndexStats,
    IndexStatus, IndexThresholds, MergeInsertStats, QueryMetrics, Recommendation,
    TableColumnAlteration, TableColumnType, TableHealthReport, TableInfo, TableNewColumn,
    TableVersionInfo,
};
