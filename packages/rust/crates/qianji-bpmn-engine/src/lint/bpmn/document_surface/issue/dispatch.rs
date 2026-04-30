use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::LintIssue;

use super::collaboration::collaboration_issue;
use super::data::data_artifact_issue;
use super::diagram::diagram_issue;

pub(super) fn issue_for_tag(
    source: &BpmnSourceFile,
    tag: &str,
    parent: Option<&str>,
) -> Option<LintIssue> {
    match tag {
        "collaboration"
        | "partnerEntity"
        | "partnerRole"
        | "participant"
        | "messageFlow"
        | "conversation"
        | "choreography"
        | "globalChoreographyTask"
        | "choreographyTask"
        | "subChoreography"
        | "callChoreography" => Some(collaboration_issue(source, tag)),
        "dataStore" | "dataStoreReference" => Some(data_artifact_issue(source, tag)),
        "ioSpecification" if parent.is_some_and(|parent| parent == "process") => {
            Some(data_artifact_issue(source, tag))
        }
        "BPMNDiagram" | "BPMNPlane" | "BPMNShape" | "BPMNEdge" | "BPMNLabel" | "BPMNLabelStyle" => {
            Some(diagram_issue(source, tag))
        }
        _ => None,
    }
}
