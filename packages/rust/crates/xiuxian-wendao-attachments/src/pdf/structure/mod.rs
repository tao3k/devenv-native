//! Document structure Arrow sidecar schema, projection, and parity API.

mod model;
mod parity;

pub use model::{
    DOCUMENT_STRUCTURE_ARROW_CACHE_NAME, DOCUMENT_STRUCTURE_SCHEMA_VERSION, DocumentStructureBlock,
    build_document_structure_batch, document_resource_batch_to_structure_blocks,
    document_structure_schema,
};
pub use parity::{
    DocumentStructureParityCount, DocumentStructureParitySummary,
    validate_document_structure_parity,
};
