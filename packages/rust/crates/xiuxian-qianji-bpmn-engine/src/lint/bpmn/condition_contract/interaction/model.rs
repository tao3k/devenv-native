use super::HashMap;

#[derive(Default)]
pub(in crate::lint::bpmn::condition_contract) struct ActiveNativeInteractionTask {
    pub(in crate::lint::bpmn::condition_contract) task_id: String,
    pub(in crate::lint::bpmn::condition_contract) data_inputs: HashMap<String, String>,
    pub(in crate::lint::bpmn::condition_contract) data_outputs: HashMap<String, String>,
    pub(in crate::lint::bpmn::condition_contract) input_associations: Vec<NativeInputAssociation>,
    pub(in crate::lint::bpmn::condition_contract) output_associations: Vec<NativeOutputAssociation>,
    pub(in crate::lint::bpmn::condition_contract) active_input_association:
        Option<NativeInputAssociation>,
    pub(in crate::lint::bpmn::condition_contract) active_output_association:
        Option<NativeOutputAssociation>,
    pub(in crate::lint::bpmn::condition_contract) text_capture: Option<NativeAssociationCapture>,
}

#[derive(Default)]
pub(in crate::lint::bpmn::condition_contract) struct NativeInputAssociation {
    pub(in crate::lint::bpmn::condition_contract) source_ref: Option<String>,
    pub(in crate::lint::bpmn::condition_contract) target_ref: Option<String>,
    pub(in crate::lint::bpmn::condition_contract) assignment_from: Option<String>,
    pub(in crate::lint::bpmn::condition_contract) assignment_to: Option<String>,
}

#[derive(Default)]
pub(in crate::lint::bpmn::condition_contract) struct NativeOutputAssociation {
    pub(in crate::lint::bpmn::condition_contract) source_ref: Option<String>,
    pub(in crate::lint::bpmn::condition_contract) target_ref: Option<String>,
}

pub(in crate::lint::bpmn::condition_contract) enum NativeAssociationCapture {
    InputSourceRef,
    InputTargetRef,
    InputAssignmentFrom,
    InputAssignmentTo,
    OutputSourceRef,
    OutputTargetRef,
}
