use super::{BTreeSet, Range, Value, json, sorted_set_values};

pub(in crate::lint::bpmn::loop_risk) fn line_fix(
    span: Option<&Range<usize>>,
    target: &str,
    xml: &str,
) -> Value {
    let mut fix = json!({
        "target": target,
        "xml": xml,
    });
    if let Some(span) = span {
        fix["offset"] = json!(span.start);
    } else {
        fix["line"] = json!("primary");
    }
    fix
}

pub(in crate::lint::bpmn::loop_risk) fn native_input_fragment(
    task_id: &str,
    inputs: &BTreeSet<String>,
) -> String {
    let mut xml = String::new();
    for input in sorted_set_values(inputs) {
        let id = stable_xml_id(task_id, "Input", &input);
        xml.push_str("<dataInput id=\"");
        xml.push_str(&id);
        xml.push_str("\" name=\"");
        xml.push_str(&input);
        xml.push_str("\"/><dataInputAssociation><sourceRef>");
        xml.push_str(&input);
        xml.push_str("</sourceRef><targetRef>");
        xml.push_str(&id);
        xml.push_str("</targetRef></dataInputAssociation>");
    }
    xml
}

pub(in crate::lint::bpmn::loop_risk) fn native_output_fragment(
    task_id: &str,
    outputs: &BTreeSet<String>,
) -> String {
    let mut xml = String::new();
    for output in sorted_set_values(outputs) {
        let id = stable_xml_id(task_id, "Output", &output);
        xml.push_str("<dataOutput id=\"");
        xml.push_str(&id);
        xml.push_str("\" name=\"");
        xml.push_str(&output);
        xml.push_str("\"/><dataOutputAssociation><sourceRef>");
        xml.push_str(&id);
        xml.push_str("</sourceRef><targetRef>");
        xml.push_str(&output);
        xml.push_str("</targetRef></dataOutputAssociation>");
    }
    xml
}

pub(in crate::lint::bpmn::loop_risk) fn stable_xml_id(
    task_id: &str,
    role: &str,
    value: &str,
) -> String {
    let mut id = format!("{task_id}_{role}_{value}")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if id
        .chars()
        .next()
        .is_none_or(|ch| !(ch.is_ascii_alphabetic() || ch == '_'))
    {
        id.insert(0, '_');
    }
    id
}
