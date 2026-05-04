//! Public BPMN document snapshot `api` facade.

mod api;
#[path = "artifact/api.rs"]
mod artifact;
#[path = "collaboration/api.rs"]
mod collaboration;
#[path = "data/api.rs"]
mod data;
#[path = "definitions/api.rs"]
mod definitions;
#[path = "di/api.rs"]
mod di;
#[path = "document/api.rs"]
mod document;
#[path = "process/api.rs"]
mod process;
#[path = "root/api.rs"]
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
