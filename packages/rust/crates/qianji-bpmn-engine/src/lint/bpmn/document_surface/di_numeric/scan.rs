use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::xml::local_name;

use super::model::DiNumericViolation;

const BPMN_DIAGRAM_NUMERIC_ATTRIBUTES: &[&str] = &["resolution"];
const DC_BOUNDS_NUMERIC_ATTRIBUTES: &[&str] = &["x", "y", "width", "height"];
const DI_WAYPOINT_NUMERIC_ATTRIBUTES: &[&str] = &["x", "y"];
const DC_FONT_NUMERIC_ATTRIBUTES: &[&str] = &["size"];

pub(super) fn collect_numeric_violations(
    source: &BpmnSourceFile,
) -> Option<Vec<DiNumericViolation>> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);
    let mut path = Vec::<String>::new();
    let mut violations = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                inspect_event(&reader, &event, &path, &mut violations);
                push_path(&event, &mut path);
            }
            Ok(Event::Empty(event)) => {
                inspect_event(&reader, &event, &path, &mut violations);
            }
            Ok(Event::End(_)) => {
                let _ = path.pop();
            }
            Ok(Event::Eof) => return Some(violations),
            Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn inspect_event(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    parent_path: &[String],
    violations: &mut Vec<DiNumericViolation>,
) {
    let event_name = event.name();
    let Some(element) = local_name(event_name.as_ref()) else {
        return;
    };
    let Some(attributes) = numeric_attributes_for_element(element) else {
        return;
    };
    for attribute_name in attributes {
        let Some(value) = attribute_value(reader, event, attribute_name) else {
            continue;
        };
        if is_valid_finite_double(&value) {
            continue;
        }
        violations.push(DiNumericViolation::new(
            element,
            attribute_value(reader, event, "id"),
            &event_path(parent_path, element),
            attribute_name,
            &value,
        ));
    }
}

fn push_path(event: &BytesStart<'_>, path: &mut Vec<String>) {
    let event_name = event.name();
    if let Some(element) = local_name(event_name.as_ref()) {
        path.push(element.to_string());
    }
}

fn event_path(parent_path: &[String], element: &str) -> String {
    let mut path = parent_path.join("/");
    if path.is_empty() {
        element.to_string()
    } else {
        path.push('/');
        path.push_str(element);
        path
    }
}

fn attribute_value(reader: &Reader<&[u8]>, event: &BytesStart<'_>, name: &str) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if local_name(attribute.key.as_ref()) == Some(name) {
            return attribute
                .decode_and_unescape_value(reader.decoder())
                .ok()
                .map(std::borrow::Cow::into_owned);
        }
    }
    None
}

fn numeric_attributes_for_element(element: &str) -> Option<&'static [&'static str]> {
    match element {
        "BPMNDiagram" => Some(BPMN_DIAGRAM_NUMERIC_ATTRIBUTES),
        "Bounds" => Some(DC_BOUNDS_NUMERIC_ATTRIBUTES),
        "waypoint" => Some(DI_WAYPOINT_NUMERIC_ATTRIBUTES),
        "Font" => Some(DC_FONT_NUMERIC_ATTRIBUTES),
        _ => None,
    }
}

fn is_valid_finite_double(value: &str) -> bool {
    value.trim().parse::<f64>().is_ok_and(f64::is_finite)
}
