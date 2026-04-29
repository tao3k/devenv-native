//! Public BPMN document snapshot `api` facade.

mod api;
#[path = "bpmn_model_api/artifact/api.rs"]
mod artifact;
#[path = "bpmn_model_api/collaboration/api.rs"]
mod collaboration;
#[path = "bpmn_model_api/data/api.rs"]
mod data;
#[path = "bpmn_model_api/definitions/api.rs"]
mod definitions;
#[path = "bpmn_model_api/di/api.rs"]
mod di;
#[path = "bpmn_model_api/document/api.rs"]
mod document;
#[path = "bpmn_model_api/process/api.rs"]
mod process;
#[path = "bpmn_model_api/root/api.rs"]
mod root;

pub(crate) use api::empty_bpmn_root_snapshot;
pub use api::{
    BpmnAssociationSnapshot, BpmnBoundsSnapshot, BpmnCategorySnapshot, BpmnCategoryValueSnapshot,
    BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot,
    BpmnConversationAssociationSnapshot, BpmnConversationLinkSnapshot,
    BpmnConversationNodeSnapshot, BpmnCorrelationKeySnapshot,
    BpmnCorrelationPropertyBindingSnapshot, BpmnCorrelationPropertySnapshot,
    BpmnCorrelationRetrievalExpressionSnapshot, BpmnCorrelationSubscriptionSnapshot,
    BpmnDataAssociationAssignmentSnapshot, BpmnDataAssociationExpressionSnapshot,
    BpmnDataAssociationSnapshot, BpmnDataInputOutputSnapshot, BpmnDataObjectReferenceSnapshot,
    BpmnDataObjectSnapshot, BpmnDataStateSnapshot, BpmnDataStoreReferenceSnapshot,
    BpmnDataStoreSnapshot, BpmnDiagramSnapshot, BpmnDocumentSnapshot, BpmnEdgeSnapshot,
    BpmnEndPointSnapshot, BpmnErrorSnapshot, BpmnEscalationSnapshot, BpmnExtensionSnapshot,
    BpmnFlowElementMetadataSnapshot, BpmnFontSnapshot, BpmnGlobalTaskSnapshot, BpmnGroupSnapshot,
    BpmnImportSnapshot, BpmnInputSetSnapshot, BpmnInterfaceSnapshot, BpmnIoBindingSnapshot,
    BpmnIoSpecificationSnapshot, BpmnItemDefinitionSnapshot, BpmnLabelSnapshot,
    BpmnLabelStyleSnapshot, BpmnLaneSetSnapshot, BpmnLaneSnapshot,
    BpmnMessageFlowAssociationSnapshot, BpmnMessageFlowSnapshot, BpmnMessageSnapshot,
    BpmnOperationSnapshot, BpmnOutputSetSnapshot, BpmnParticipantAssociationSnapshot,
    BpmnParticipantMultiplicitySnapshot, BpmnParticipantSnapshot, BpmnPartnerEntitySnapshot,
    BpmnPartnerRoleSnapshot, BpmnPlaneSnapshot, BpmnProcessPropertySnapshot, BpmnProcessSnapshot,
    BpmnRelationshipSnapshot, BpmnResourceParameterBindingSnapshot, BpmnResourceParameterSnapshot,
    BpmnResourceRoleSnapshot, BpmnResourceSnapshot, BpmnRootSnapshot, BpmnShapeSnapshot,
    BpmnSignalSnapshot, BpmnTextAnnotationSnapshot, BpmnWaypointSnapshot,
};
