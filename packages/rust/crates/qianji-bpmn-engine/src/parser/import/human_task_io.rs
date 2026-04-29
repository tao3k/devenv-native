use super::attributes::attribute_value;
use super::capture::last_process_node_mut;
use super::model::{
    CaptureTarget, RawHumanTaskChoiceSpec, RawHumanTaskFormSpec, RawHumanTaskFreeTextSpec,
    RawHumanTaskIoAssociation, RawHumanTaskIoAssociationKind, RawHumanTaskIoDeclaration,
    RawHumanTaskIoDeclarationKind, RawHumanTaskNativeIoSpec, RawProcess,
};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::error::{BpmnEngineError, Result};
use crate::ir_node_api::BpmnNodeKind;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_human_task_io_child_start(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    tag: &str,
    parent: &str,
    process: &mut RawProcess,
    capture_target: &mut Option<CaptureTarget>,
    capture_buffer: &mut String,
    is_empty: bool,
) -> Result<bool> {
    HumanTaskIoStartContext {
        source,
        reader,
        event,
        tag,
        parent,
        process,
        capture_target,
        capture_buffer,
        is_empty,
    }
    .handle()
}

struct HumanTaskIoStartContext<'a, 'reader, 'event> {
    source: &'a BpmnSourceFile,
    reader: &'a Reader<&'reader [u8]>,
    event: &'a BytesStart<'event>,
    tag: &'a str,
    parent: &'a str,
    process: &'a mut RawProcess,
    capture_target: &'a mut Option<CaptureTarget>,
    capture_buffer: &'a mut String,
    is_empty: bool,
}

impl HumanTaskIoStartContext<'_, '_, '_> {
    fn handle(&mut self) -> Result<bool> {
        if self.handle_documentation_start()? || self.handle_io_container_start()? {
            return Ok(true);
        }
        match self.parent {
            "ioSpecification" => self.handle_io_specification_child_start(),
            "inputSet" | "outputSet" => Ok(matches!(self.tag, "dataInputRefs" | "dataOutputRefs")),
            "dataInputAssociation" | "dataOutputAssociation" => {
                self.handle_association_child_start()
            }
            "assignment" => self.handle_assignment_child_start(),
            _ => Ok(false),
        }
    }

    fn handle_documentation_start(&mut self) -> Result<bool> {
        if !is_human_task(self.parent) || self.tag != "documentation" {
            return Ok(false);
        }
        self.capture_text_start(
            CaptureTarget::HumanTaskDocumentationText,
            apply_human_task_documentation_text,
        )
    }

    fn handle_io_container_start(&mut self) -> Result<bool> {
        if !is_supported_task(self.parent)
            || !matches!(
                self.tag,
                "ioSpecification" | "dataInputAssociation" | "dataOutputAssociation"
            )
        {
            return Ok(false);
        }
        if is_human_task(self.parent) {
            match self.tag {
                "dataInputAssociation" => {
                    start_association(
                        self.source,
                        self.process,
                        RawHumanTaskIoAssociationKind::DataInput,
                        self.is_empty,
                    )?;
                }
                "dataOutputAssociation" => {
                    start_association(
                        self.source,
                        self.process,
                        RawHumanTaskIoAssociationKind::DataOutput,
                        self.is_empty,
                    )?;
                }
                _ => {}
            }
        }
        Ok(true)
    }

    fn handle_io_specification_child_start(&mut self) -> Result<bool> {
        if !last_node_is_human_task(self.process) {
            return Ok(matches!(
                self.tag,
                "dataInput" | "dataOutput" | "inputSet" | "outputSet"
            ));
        }
        match self.tag {
            "dataInput" => self.record_declaration(RawHumanTaskIoDeclarationKind::DataInput),
            "dataOutput" => self.record_declaration(RawHumanTaskIoDeclarationKind::DataOutput),
            "inputSet" | "outputSet" => Ok(true),
            _ => Ok(false),
        }
    }

    fn record_declaration(&mut self, kind: RawHumanTaskIoDeclarationKind) -> Result<bool> {
        record_declaration(self.source, self.reader, self.event, self.process, kind)?;
        Ok(true)
    }

    fn handle_association_child_start(&mut self) -> Result<bool> {
        if !last_node_is_human_task(self.process) {
            return Ok(matches!(
                self.tag,
                "sourceRef" | "targetRef" | "assignment" | "transformation"
            ));
        }
        match self.tag {
            "sourceRef" => self.capture_text_start(
                CaptureTarget::HumanTaskIoSourceRef,
                apply_human_task_io_source_ref,
            ),
            "targetRef" => self.capture_text_start(
                CaptureTarget::HumanTaskIoTargetRef,
                apply_human_task_io_target_ref,
            ),
            "assignment" | "transformation" => Ok(true),
            _ => Ok(false),
        }
    }

    fn handle_assignment_child_start(&mut self) -> Result<bool> {
        if !last_node_is_human_task(self.process) {
            return Ok(matches!(self.tag, "from" | "to"));
        }
        match self.tag {
            "from" => self.capture_text_start(
                CaptureTarget::HumanTaskIoAssignmentFrom,
                apply_human_task_io_assignment_from,
            ),
            "to" => self.capture_text_start(
                CaptureTarget::HumanTaskIoAssignmentTo,
                apply_human_task_io_assignment_to,
            ),
            _ => Ok(false),
        }
    }

    fn capture_text_start(
        &mut self,
        target: CaptureTarget,
        apply_empty: fn(&BpmnSourceFile, &mut RawProcess, &str) -> Result<()>,
    ) -> Result<bool> {
        *self.capture_target = Some(target);
        self.capture_buffer.clear();
        if self.is_empty {
            apply_empty(self.source, self.process, "")?;
            *self.capture_target = None;
            self.capture_buffer.clear();
        }
        Ok(true)
    }
}

pub(super) fn apply_human_task_documentation_text(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let io = ensure_native_io(source, process)?;
    match &mut io.documentation_text {
        Some(existing) if !existing.is_empty() => {
            existing.push(' ');
            existing.push_str(text);
        }
        _ => io.documentation_text = Some(text.to_string()),
    }
    sync_form_from_native_io(source, process)
}

pub(super) fn apply_human_task_io_source_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.source_refs.push(text.to_string());
    Ok(())
}

pub(super) fn apply_human_task_io_target_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.target_ref = Some(text.to_string());
    Ok(())
}

pub(super) fn apply_human_task_io_assignment_from(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.assignment_from = Some(text.to_string());
    Ok(())
}

pub(super) fn apply_human_task_io_assignment_to(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    text: &str,
) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let association = active_association_mut(source, process)?;
    association.assignment_to = Some(text.to_string());
    Ok(())
}

pub(super) fn complete_human_task_io_end_tag(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    tag: &str,
) -> Result<()> {
    let expected_kind = match tag {
        "dataInputAssociation" => RawHumanTaskIoAssociationKind::DataInput,
        "dataOutputAssociation" => RawHumanTaskIoAssociationKind::DataOutput,
        _ => return Ok(()),
    };
    if !last_node_is_human_task(process) {
        return Ok(());
    }
    let Some(association) = native_io_mut(source, process)?.active_association.take() else {
        return Ok(());
    };
    if association.kind != expected_kind {
        return Ok(());
    }
    match association.kind {
        RawHumanTaskIoAssociationKind::DataInput => apply_input_association(
            source,
            process,
            association
                .target_ref
                .as_deref()
                .or(association.assignment_to.as_deref()),
            association.source_refs.first().map(String::as_str),
            association.assignment_from.as_deref(),
        )?,
        RawHumanTaskIoAssociationKind::DataOutput => apply_output_association(
            source,
            process,
            association
                .source_refs
                .first()
                .map(String::as_str)
                .or(association.assignment_from.as_deref()),
            association.target_ref.or(association.assignment_to),
        )?,
    }
    sync_form_from_native_io(source, process)
}

fn record_declaration(
    source: &BpmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    process: &mut RawProcess,
    kind: RawHumanTaskIoDeclarationKind,
) -> Result<()> {
    if !last_node_is_human_task(process) {
        return Ok(());
    }
    let Some(id) = attribute_value(reader, event, "id")? else {
        return Ok(());
    };
    let Some(name) = attribute_value(reader, event, "name")? else {
        return Ok(());
    };
    let io = ensure_native_io(source, process)?;
    io.declarations
        .push(RawHumanTaskIoDeclaration { id, name, kind });
    Ok(())
}

fn start_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    kind: RawHumanTaskIoAssociationKind,
    is_empty: bool,
) -> Result<()> {
    let io = ensure_native_io(source, process)?;
    io.active_association = Some(RawHumanTaskIoAssociation::new(kind));
    if is_empty {
        io.active_association = None;
    }
    Ok(())
}

fn apply_input_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    target_ref: Option<&str>,
    source_ref: Option<&str>,
    literal: Option<&str>,
) -> Result<()> {
    let Some(target_ref) = target_ref else {
        return Ok(());
    };
    let Some(name) = input_name_for_ref(source, process, target_ref)? else {
        return Ok(());
    };
    let io = native_io_mut(source, process)?;
    match name.as_str() {
        "interactionType" => {
            if let Some(value) = literal.map(str::trim).filter(|value| !value.is_empty()) {
                io.interaction_type = Some(value.to_string());
            }
        }
        "question" => {
            if let Some(source_ref) = source_ref {
                io.question_ref = Some(source_ref.to_string());
                io.question_text = None;
            } else if let Some(value) = literal.map(str::trim).filter(|value| !value.is_empty()) {
                io.question_text = Some(value.to_string());
                io.question_ref = None;
            }
        }
        "choices" => {
            if let Some(source_ref) = source_ref {
                io.choices_ref = Some(source_ref.to_string());
            } else if let Some(value) = literal {
                io.choices = parse_choice_literal(value)?;
            }
        }
        "freeText" => {
            if let Some(value) = literal {
                io.free_text_fields = parse_free_text_literal(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn apply_output_association(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    source_ref: Option<&str>,
    target_ref: Option<String>,
) -> Result<()> {
    let Some(target_ref) = target_ref else {
        return Ok(());
    };
    let output_name = match source_ref {
        Some(source_ref) => output_name_for_ref(source, process, source_ref)?,
        None => None,
    };
    if output_name.as_deref() == Some("answer") || output_name.is_none() {
        native_io_mut(source, process)?.result_output = Some(target_ref);
    }
    Ok(())
}

fn input_name_for_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    target_ref: &str,
) -> Result<Option<String>> {
    Ok(native_io_mut(source, process)?
        .declarations
        .iter()
        .find(|declaration| {
            declaration.kind == RawHumanTaskIoDeclarationKind::DataInput
                && declaration.id == target_ref
        })
        .map(|declaration| declaration.name.clone()))
}

fn output_name_for_ref(
    source: &BpmnSourceFile,
    process: &mut RawProcess,
    source_ref: &str,
) -> Result<Option<String>> {
    Ok(native_io_mut(source, process)?
        .declarations
        .iter()
        .find(|declaration| {
            declaration.kind == RawHumanTaskIoDeclarationKind::DataOutput
                && declaration.id == source_ref
        })
        .map(|declaration| declaration.name.clone()))
}

fn sync_form_from_native_io(source: &BpmnSourceFile, process: &mut RawProcess) -> Result<()> {
    let io = native_io_mut(source, process)?.clone();
    let Some(interaction_type) = io.interaction_type else {
        return Ok(());
    };
    let question_text = if io.question_ref.is_some() {
        io.question_text
    } else {
        io.question_text.or(io.documentation_text)
    };
    last_process_node_mut(source, process)?.human_task_form = Some(RawHumanTaskFormSpec {
        interaction_type,
        question_ref: io.question_ref,
        question_text,
        choices_ref: io.choices_ref,
        choices: io.choices,
        free_text_fields: io.free_text_fields,
        result_output: io.result_output,
    });
    Ok(())
}

fn parse_choice_literal(value: &str) -> Result<Vec<RawHumanTaskChoiceSpec>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_choices_json",
        })?;
    let Some(items) = parsed.as_array() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_choices_json",
        });
    };
    items
        .iter()
        .map(|item| {
            if let Some(value) = item.as_str() {
                return Ok(RawHumanTaskChoiceSpec {
                    value: value.to_string(),
                    label: None,
                });
            }
            let Some(object) = item.as_object() else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "invalid_native_human_task_choices_json",
                });
            };
            let Some(value) = object.get("value").and_then(|value| value.as_str()) else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "invalid_native_human_task_choices_json",
                });
            };
            Ok(RawHumanTaskChoiceSpec {
                value: value.to_string(),
                label: object
                    .get("label")
                    .and_then(|label| label.as_str())
                    .map(ToString::to_string),
            })
        })
        .collect()
}

fn parse_free_text_literal(value: &str) -> Result<Vec<RawHumanTaskFreeTextSpec>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_free_text_json",
        })?;
    match parsed {
        serde_json::Value::String(name) => Ok(vec![RawHumanTaskFreeTextSpec {
            name,
            optional: false,
        }]),
        serde_json::Value::Object(object) => Ok(vec![free_text_from_object(&object)?]),
        serde_json::Value::Array(items) => items
            .iter()
            .map(|item| {
                let Some(object) = item.as_object() else {
                    return Err(BpmnEngineError::UnsupportedOperation {
                        operation: "invalid_native_human_task_free_text_json",
                    });
                };
                free_text_from_object(object)
            })
            .collect(),
        _ => Err(BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_free_text_json",
        }),
    }
}

fn free_text_from_object(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<RawHumanTaskFreeTextSpec> {
    let Some(name) = object.get("name").and_then(|value| value.as_str()) else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "invalid_native_human_task_free_text_json",
        });
    };
    Ok(RawHumanTaskFreeTextSpec {
        name: name.to_string(),
        optional: object
            .get("optional")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    })
}

fn active_association_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawHumanTaskIoAssociation> {
    native_io_mut(source, process)?
        .active_association
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "native_human_task_io_association_child_without_association",
        })
}

fn ensure_native_io<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawHumanTaskNativeIoSpec> {
    let node = last_process_node_mut(source, process)?;
    if !matches!(node.kind, BpmnNodeKind::UserTask | BpmnNodeKind::ManualTask) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "native_human_task_io_without_human_task",
        });
    }
    if node.native_human_task_io.is_none() {
        node.native_human_task_io = Some(RawHumanTaskNativeIoSpec::default());
    }
    node.native_human_task_io
        .as_mut()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "native_human_task_io_missing_state",
        })
}

fn native_io_mut<'a>(
    source: &BpmnSourceFile,
    process: &'a mut RawProcess,
) -> Result<&'a mut RawHumanTaskNativeIoSpec> {
    ensure_native_io(source, process)
}

fn last_node_is_human_task(process: &RawProcess) -> bool {
    process
        .nodes
        .last()
        .is_some_and(|node| matches!(node.kind, BpmnNodeKind::UserTask | BpmnNodeKind::ManualTask))
}

fn is_human_task(tag: &str) -> bool {
    matches!(tag, "userTask" | "manualTask")
}

fn is_supported_task(tag: &str) -> bool {
    matches!(
        tag,
        "serviceTask"
            | "userTask"
            | "manualTask"
            | "businessRuleTask"
            | "scriptTask"
            | "sendTask"
            | "receiveTask"
            | "task"
    )
}
