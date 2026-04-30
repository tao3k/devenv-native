use super::api::{
    BpmnCallActivityBinding, BpmnCallableBindingExecutionPolicy, BpmnCallableDataRef,
    BpmnCallableDefinition, BpmnCallableIoBinding, BpmnCallableKind, BpmnCallableRegistry,
};
use crate::bpmn_model_api::{
    BpmnDataInputOutputSnapshot, BpmnDocumentSnapshot, BpmnGlobalTaskSnapshot,
    BpmnIoBindingSnapshot, BpmnIoSpecificationSnapshot, BpmnProcessSnapshot,
};
use crate::ir_node_api::BpmnSubProcessKind;
use crate::ir_package_api::BpmnPackage;
use std::sync::Arc;

impl BpmnCallableRegistry {
    /// Builds a callable registry from one document snapshot plus the
    /// executable package that was normalized from that same document.
    #[must_use]
    pub(crate) fn from_document_snapshot(
        snapshot: &BpmnDocumentSnapshot,
        package: &BpmnPackage,
    ) -> Self {
        let definitions = callable_definitions_from_snapshot(snapshot, package);
        let call_activity_bindings = call_activity_bindings_from_package(package, &definitions);
        Self {
            definitions,
            call_activity_bindings,
        }
    }
}

fn callable_definitions_from_snapshot(
    snapshot: &BpmnDocumentSnapshot,
    package: &BpmnPackage,
) -> Vec<BpmnCallableDefinition> {
    let mut definitions = Vec::new();
    definitions.extend(
        snapshot
            .processes
            .iter()
            .filter_map(|process| process_callable_definition(snapshot, package, process)),
    );
    definitions.extend(
        snapshot
            .root
            .global_tasks
            .iter()
            .filter_map(|task| global_task_callable_definition(snapshot, task)),
    );
    definitions
}

fn process_callable_definition(
    snapshot: &BpmnDocumentSnapshot,
    package: &BpmnPackage,
    process: &BpmnProcessSnapshot,
) -> Option<BpmnCallableDefinition> {
    let process_id = process.process_id.as_ref()?;
    Some(BpmnCallableDefinition {
        callable_id: Arc::<str>::from(process_id.as_str()),
        kind: BpmnCallableKind::Process,
        name: optional_arc(process.name.as_deref()),
        source_id: Arc::<str>::from(snapshot.source_id.as_str()),
        is_executable: process.is_executable,
        runtime_available: package.find_process(process_id).is_some(),
        process_type: optional_arc(process.process_type.as_deref()),
        is_closed: process.is_closed,
        implementation: None,
        script_language: None,
        script: None,
        supported_interface_refs: arc_vec(&process.supports),
        inputs: callable_inputs(&process.io_specifications),
        outputs: callable_outputs(&process.io_specifications),
        io_bindings: callable_io_bindings(&process.io_bindings),
    })
}

fn global_task_callable_definition(
    snapshot: &BpmnDocumentSnapshot,
    task: &BpmnGlobalTaskSnapshot,
) -> Option<BpmnCallableDefinition> {
    let task_id = task.task_id.as_ref()?;
    Some(BpmnCallableDefinition {
        callable_id: Arc::<str>::from(task_id.as_str()),
        kind: BpmnCallableKind::from_global_task_tag(&task.task_kind)?,
        name: optional_arc(task.name.as_deref()),
        source_id: Arc::<str>::from(snapshot.source_id.as_str()),
        is_executable: None,
        runtime_available: false,
        process_type: None,
        is_closed: None,
        implementation: optional_arc(task.implementation.as_deref()),
        script_language: optional_arc(task.script_language.as_deref()),
        script: optional_arc(task.script.as_deref()),
        supported_interface_refs: arc_vec(&task.supported_interface_refs),
        inputs: callable_inputs(&task.io_specifications),
        outputs: callable_outputs(&task.io_specifications),
        io_bindings: callable_io_bindings(&task.io_bindings),
    })
}

fn call_activity_bindings_from_package(
    package: &BpmnPackage,
    definitions: &[BpmnCallableDefinition],
) -> Vec<BpmnCallActivityBinding> {
    let mut bindings = Vec::new();
    for process in &package.processes {
        for node in &process.nodes {
            if node.subprocess_kind != Some(BpmnSubProcessKind::CallActivity) {
                continue;
            }
            let Some(target_id) = node.called_process_id.as_ref() else {
                continue;
            };
            let Some(target) = definitions
                .iter()
                .find(|definition| definition.callable_id.as_ref() == target_id.as_ref())
            else {
                continue;
            };
            if target.kind != BpmnCallableKind::Process || !target.runtime_available {
                continue;
            }
            bindings.push(BpmnCallActivityBinding {
                process_id: Arc::clone(&process.key.process_id),
                activity_id: Arc::clone(&node.bpmn_id),
                target_id: Arc::clone(&target.callable_id),
                target_kind: target.kind,
                execution_policy: BpmnCallableBindingExecutionPolicy::BoundedProcessCall,
            });
        }
    }
    bindings
}

fn callable_inputs(specifications: &[BpmnIoSpecificationSnapshot]) -> Vec<BpmnCallableDataRef> {
    specifications
        .iter()
        .flat_map(|specification| specification.data_inputs.iter())
        .map(callable_data_ref)
        .collect()
}

fn callable_outputs(specifications: &[BpmnIoSpecificationSnapshot]) -> Vec<BpmnCallableDataRef> {
    specifications
        .iter()
        .flat_map(|specification| specification.data_outputs.iter())
        .map(callable_data_ref)
        .collect()
}

fn callable_data_ref(data: &BpmnDataInputOutputSnapshot) -> BpmnCallableDataRef {
    BpmnCallableDataRef {
        data_id: optional_arc(data.data_id.as_deref()),
        name: optional_arc(data.name.as_deref()),
        item_subject_ref: optional_arc(data.item_subject_ref.as_deref()),
        is_collection: data.is_collection,
    }
}

fn callable_io_bindings(bindings: &[BpmnIoBindingSnapshot]) -> Vec<BpmnCallableIoBinding> {
    bindings
        .iter()
        .map(|binding| BpmnCallableIoBinding {
            binding_id: optional_arc(binding.binding_id.as_deref()),
            operation_ref: optional_arc(binding.operation_ref.as_deref()),
            input_data_ref: optional_arc(binding.input_data_ref.as_deref()),
            output_data_ref: optional_arc(binding.output_data_ref.as_deref()),
        })
        .collect()
}

fn optional_arc(value: Option<&str>) -> Option<Arc<str>> {
    value.map(Arc::<str>::from)
}

fn arc_vec(values: &[String]) -> Vec<Arc<str>> {
    values
        .iter()
        .map(|value| Arc::<str>::from(value.as_str()))
        .collect()
}
