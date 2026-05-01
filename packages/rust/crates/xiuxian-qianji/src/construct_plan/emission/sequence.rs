use super::ids::{flow_id, gateway_id, node_ref};
use super::xml::push_xml;
use crate::construct_plan::api::{WorkflowPlanEdge, escape_xml_text};

pub(super) fn push_edge_xml(
    xml: &mut String,
    gateway_sources: &[&str],
    edge: &WorkflowPlanEdge,
    index: usize,
) {
    let flow_id = flow_id(index);
    let source_ref = if gateway_sources.contains(&edge.from.as_str()) {
        gateway_id(&edge.from)
    } else {
        node_ref(&edge.from)
    };
    let target_ref = node_ref(&edge.to);
    push_sequence_flow_xml(
        xml,
        &flow_id,
        &source_ref,
        &target_ref,
        edge.condition.as_deref(),
    );
}

pub(super) fn push_sequence_flow_xml(
    xml: &mut String,
    flow_id: &str,
    source_ref: &str,
    target_ref: &str,
    condition: Option<&str>,
) {
    if let Some(condition) = condition {
        push_xml(
            xml,
            format_args!(
                "    <sequenceFlow id=\"{flow_id}\" sourceRef=\"{source_ref}\" targetRef=\"{target_ref}\">\n"
            ),
        );
        push_xml(
            xml,
            format_args!(
                "      <conditionExpression xsi:type=\"tFormalExpression\">{}</conditionExpression>\n",
                escape_xml_text(condition)
            ),
        );
        xml.push_str("    </sequenceFlow>\n");
    } else {
        push_xml(
            xml,
            format_args!(
                "    <sequenceFlow id=\"{flow_id}\" sourceRef=\"{source_ref}\" targetRef=\"{target_ref}\"/>\n"
            ),
        );
    }
}
