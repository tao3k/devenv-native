pub(super) const INSPECTION_SCHEMA: &str =
    "xiuxian_wendao.sql.candidate_read_model_duckdb_inspection.v1";
pub(super) const EXECUTION_ENGINE: &str = "duckdb";
pub(super) const REGISTRATION_STRATEGY: &str = "duckdb_read_parquet_view";
pub(super) const REVIEW_STATUS: &str = "review_required";
pub(super) const PROMOTION_STATUS: &str = "blocked_pending_review";
pub(super) const OBJECTS_PARQUET: &str = "ontology_candidate_objects.parquet";
pub(super) const RELATIONS_PARQUET: &str = "ontology_candidate_relations.parquet";
pub(super) const EVIDENCE_PARQUET: &str = "ontology_candidate_evidence.parquet";
