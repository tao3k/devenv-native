//! DuckDB-backed BPMN workflow data-store adapter.

mod config;
mod error;
mod record;
mod state;
mod store;

pub use config::{DEFAULT_QIANJI_BPMN_DUCKDB_THREADS, QianjiBpmnDuckDbDataStoreConfig};
pub use error::QianjiBpmnDataStoreError;
pub use record::QianjiBpmnDataRecord;
pub use state::QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY;
pub use store::QianjiBpmnDuckDbDataStore;
