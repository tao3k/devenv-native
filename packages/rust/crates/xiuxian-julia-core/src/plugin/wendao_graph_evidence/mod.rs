//! Rust mirror of `WendaoGraph.jl` evidence table contracts.

mod contracts;
mod names;
mod page_index_columns;
mod request_columns;
mod response_columns;
mod route;
mod schema;
mod types;

pub use contracts::{
    WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS, WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_CONTRACTS,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_CONTRACTS,
};
pub use names::{
    WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_NAMES, WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_NAMES,
    WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_NAMES,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_NAMES,
};
pub use route::{is_wendao_graph_link_evidence_route, wendao_graph_link_evidence_route};
pub use schema::{
    validate_wendao_graph_evidence_request_schema, validate_wendao_graph_evidence_response_schema,
    validate_wendao_graph_page_index_reasoning_request_schema,
    validate_wendao_graph_page_index_reasoning_response_schema,
    wendao_graph_evidence_request_table_contract, wendao_graph_evidence_response_table_contract,
    wendao_graph_evidence_table_schema, wendao_graph_page_index_reasoning_request_table_contract,
    wendao_graph_page_index_reasoning_response_table_contract,
    wendao_graph_page_index_reasoning_table_schema,
};
pub use types::{
    WendaoGraphEvidenceColumnContract, WendaoGraphEvidenceColumnType,
    WendaoGraphEvidenceTableContract, WendaoGraphEvidenceTableKind,
};

#[cfg(test)]
#[path = "../../../tests/unit/plugin/wendao_graph_evidence/mod.rs"]
mod tests;
