//! Episteme Gateway admission handlers.

mod model;
mod planning;
mod registry;

#[cfg(all(test, feature = "julia"))]
pub(crate) use model::EpistemeOntologyRegistryReadModelGatewayReport;
pub(crate) use model::{
    EPISTEME_EVIDENCE_READ_ROUTE, EPISTEME_EVIDENCE_SELECTION_PLAN_ROUTE,
    EPISTEME_ONTOLOGY_REGISTRY_READ_MODEL_ROUTE, EPISTEME_SOURCE_CONTRACT_RUN_PLAN_ROUTE,
};
#[cfg(test)]
pub(crate) use model::{
    EpistemeOntologyRegistryQualityProofModeRequest,
    EpistemeOntologyRegistryReadModelGatewayRequest, EpistemeRunPlanAdmissionRequest,
};
#[cfg(test)]
pub(crate) use planning::plan_episteme_extraction_run_from_payload;
pub(crate) use planning::{
    plan_episteme_extraction_run, read_episteme_evidence_source,
    write_episteme_evidence_selection_plan_source,
};
pub(crate) use registry::admit_episteme_ontology_registry_read_model;
#[cfg(test)]
pub(crate) use registry::admit_episteme_ontology_registry_read_model_from_payload;
#[cfg(test)]
pub(crate) use registry::admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof;

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/episteme/source_contract.rs"]
mod tests;
