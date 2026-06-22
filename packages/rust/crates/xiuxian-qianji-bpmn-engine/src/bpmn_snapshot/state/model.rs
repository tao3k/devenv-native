pub(super) use crate::bpmn_model_api::{
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
pub(super) use crate::bpmn_parse_api::BpmnSourceFile;
pub(super) use crate::error::Result;
pub(super) use quick_xml::Reader;
pub(super) use quick_xml::events::BytesStart;

#[derive(Debug, Clone, Copy)]
pub(in crate::bpmn_snapshot) enum TextTarget {
    LaneFlowNode,
    DataAssociationSource,
    DataAssociationTarget,
    DataAssociationTransformation,
    DataAssociationAssignmentFrom,
    DataAssociationAssignmentTo,
    CorrelationMessagePath,
    CorrelationBindingDataPath,
    ResourceRoleResourceRef,
    ResourceRoleAssignmentExpression,
    ResourceRoleParameterBindingExpression,
    FlowElementCategoryValueRef,
    OperationInMessageRef,
    OperationOutMessageRef,
    OperationErrorRef,
    IoInputSetDataInputRef,
    IoInputSetOptionalInputRef,
    IoInputSetWhileExecutingInputRef,
    IoInputSetOutputSetRef,
    IoOutputSetDataOutputRef,
    IoOutputSetOptionalOutputRef,
    IoOutputSetWhileExecutingOutputRef,
    IoOutputSetInputSetRef,
    ExtensionDocumentation,
    RelationshipSource,
    RelationshipTarget,
    ParticipantInterfaceRef,
    ParticipantEndPointRef,
    PartnerEntityParticipantRef,
    PartnerRoleParticipantRef,
    GlobalTaskSupportedInterfaceRef,
    GlobalTaskScript,
    ProcessSupport,
    ConversationParticipantRef,
    ConversationMessageFlowRef,
    ChoreographyParticipantRef,
    ChoreographyMessageFlowRef,
    TextAnnotationText,
    CorrelationKeyPropertyRef,
    ParticipantAssociationInnerRef,
    ParticipantAssociationOuterRef,
    CollaborationChoreographyRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DataAssociationKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DataAssociationAssignmentExpressionKind {
    From,
    To,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IoSetKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BpmnDiLabelTarget {
    Shape(usize, usize),
    Edge(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollaborationMetadataOwner {
    Collaboration(usize),
    ConversationNode(usize, Vec<usize>),
    ChoreographyActivity(usize, Vec<usize>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArtifactMetadataOwner {
    Collaboration(usize),
    Process(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResourceRoleOwner {
    Process(usize),
    GlobalTask(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IoSpecificationOwner {
    Process(usize, usize),
    GlobalTask(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DataStateOwner {
    RootDataStore(usize),
    ProcessDataObject(usize, usize),
    ProcessDataObjectReference(usize, usize),
    ProcessDataStoreReference(usize, usize),
    IoDataInput(IoSpecificationOwner, usize),
    IoDataOutput(IoSpecificationOwner, usize),
}

#[derive(Debug, Default)]
pub(in crate::bpmn_snapshot) struct BpmnSnapshotScanState {
    pub(super) root: Option<BpmnRootSnapshot>,
    pub(super) collaborations: Vec<BpmnCollaborationSnapshot>,
    pub(super) processes: Vec<BpmnProcessSnapshot>,
    pub(super) current_collaboration: Option<usize>,
    pub(super) current_participant: Option<(usize, usize)>,
    pub(super) conversation_node_stack: Vec<(usize, Vec<usize>)>,
    pub(super) choreography_activity_stack: Vec<(usize, Vec<usize>)>,
    pub(super) current_conversation_correlation_key:
        Option<(CollaborationMetadataOwner, BpmnCorrelationKeySnapshot)>,
    pub(super) current_participant_association: Option<(
        CollaborationMetadataOwner,
        BpmnParticipantAssociationSnapshot,
    )>,
    pub(super) current_text_annotation: Option<(ArtifactMetadataOwner, BpmnTextAnnotationSnapshot)>,
    pub(super) current_process: Option<usize>,
    pub(super) current_correlation_subscription: Option<(usize, usize)>,
    pub(super) current_correlation_property_binding: Option<(usize, usize, usize)>,
    pub(super) current_resource_role: Option<(ResourceRoleOwner, usize)>,
    pub(super) current_resource_parameter_binding: Option<(ResourceRoleOwner, usize, usize)>,
    pub(super) current_resource_assignment_expression: Option<(ResourceRoleOwner, usize)>,
    pub(super) current_flow_element_metadata: Option<(usize, BpmnFlowElementMetadataSnapshot)>,
    pub(super) collecting_flow_element_category_value_ref: bool,
    pub(super) lane_set_stack: Vec<(usize, usize)>,
    pub(super) lane_stack: Vec<(usize, usize, usize)>,
    pub(super) current_correlation_property: Option<usize>,
    pub(super) current_correlation_retrieval_expression:
        Option<(usize, BpmnCorrelationRetrievalExpressionSnapshot)>,
    pub(super) current_partner_entity: Option<usize>,
    pub(super) current_partner_role: Option<usize>,
    pub(super) current_global_task: Option<usize>,
    pub(super) current_interface: Option<usize>,
    pub(super) current_operation: Option<(usize, usize)>,
    pub(super) current_resource: Option<usize>,
    pub(super) current_category: Option<usize>,
    pub(super) current_extension: Option<usize>,
    pub(super) current_extension_documentation: Option<(usize, String)>,
    pub(super) current_relationship: Option<usize>,
    pub(super) current_diagram: Option<usize>,
    pub(super) current_plane: Option<usize>,
    pub(super) current_shape: Option<(usize, usize)>,
    pub(super) current_edge: Option<(usize, usize)>,
    pub(super) current_label: Option<BpmnDiLabelTarget>,
    pub(super) current_label_style: Option<(usize, usize)>,
    pub(super) io_specification_stack: Vec<IoSpecificationOwner>,
    pub(super) current_io_set: Option<(IoSetKind, usize)>,
    pub(super) current_data_state_owner: Option<DataStateOwner>,
    pub(super) current_data_association:
        Option<(usize, DataAssociationKind, BpmnDataAssociationSnapshot)>,
    pub(super) current_data_association_assignment: Option<usize>,
}
