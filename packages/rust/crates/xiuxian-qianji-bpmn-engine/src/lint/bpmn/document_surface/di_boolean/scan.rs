use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::xml::local_name;

use super::model::DiBooleanViolation;

const BPMN_SHAPE_BOOLEAN_ATTRIBUTES: &[&str] = &[
    "isHorizontal",
    "isExpanded",
    "isMarkerVisible",
    "isMessageVisible",
];

const DC_FONT_BOOLEAN_ATTRIBUTES: &[&str] =
    &["isBold", "isItalic", "isUnderline", "isStrikeThrough"];

pub(super) fn collect_boolean_violations(
    source: &BpmnSourceFile,
) -> Option<Vec<DiBooleanViolation>> {
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
    violations: &mut Vec<DiBooleanViolation>,
) {
    let event_name = event.name();
    let Some(element) = local_name(event_name.as_ref()) else {
        return;
    };
    let Some(attributes) = boolean_attributes_for_element(element) else {
        return;
    };
    for attribute_name in attributes {
        let Some(value) = attribute_value(reader, event, attribute_name) else {
            continue;
        };
        if is_valid_xml_boolean(&value) {
            continue;
        }
        violations.push(DiBooleanViolation::new(
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

fn boolean_attributes_for_element(element: &str) -> Option<&'static [&'static str]> {
    match element {
        "BPMNShape" => Some(BPMN_SHAPE_BOOLEAN_ATTRIBUTES),
        "Font" => Some(DC_FONT_BOOLEAN_ATTRIBUTES),
        _ => None,
    }
}

fn is_valid_xml_boolean(value: &str) -> bool {
    matches!(value, "true" | "false" | "1" | "0")
}
