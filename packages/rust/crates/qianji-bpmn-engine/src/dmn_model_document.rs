//! Public `api` facade for DMN document snapshots.

mod api;
#[path = "dmn_model_document/core/api.rs"]
mod core;
#[path = "dmn_model_document/decision/api.rs"]
mod decision;
#[path = "dmn_model_document/dmndi/api.rs"]
mod dmndi;
#[path = "dmn_model_document/document/api.rs"]
mod document;
#[path = "dmn_model_document/function/api.rs"]
mod function;
#[path = "dmn_model_document/invocation/api.rs"]
mod invocation;
#[path = "dmn_model_document/root/api.rs"]
mod root;

pub use api::{
    DmnAssociationSnapshot, DmnBoundsSnapshot, DmnBusinessKnowledgeModelLiteralSnapshot,
    DmnBusinessKnowledgeModelSnapshot, DmnDecisionServiceDividerLineSnapshot,
    DmnDecisionServiceReferenceSnapshot, DmnDecisionServiceSnapshot, DmnDecisionSnapshot,
    DmnDiagramSnapshot, DmnDmndiSnapshot, DmnDocumentSnapshot, DmnEdgeSnapshot,
    DmnElementCollectionSnapshot, DmnFunctionDefinitionLiteralSnapshot,
    DmnFunctionDefinitionParameterSnapshot, DmnFunctionDefinitionSnapshot, DmnGroupSnapshot,
    DmnImportSnapshot, DmnInputDataSnapshot, DmnInvocationBindingSnapshot,
    DmnInvocationLiteralSnapshot, DmnInvocationParameterSnapshot, DmnInvocationSnapshot,
    DmnItemComponentSnapshot, DmnItemDefinitionSnapshot, DmnKnowledgeSourceSnapshot,
    DmnLabelSnapshot, DmnOrganizationUnitSnapshot, DmnPerformanceIndicatorSnapshot,
    DmnRequirementReferenceSnapshot, DmnRootSnapshot, DmnShapeSnapshot, DmnTextAnnotationSnapshot,
    DmnVariableSnapshot, DmnWaypointSnapshot,
};
