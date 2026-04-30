use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::xml::local_name;

use super::model::DiRequiredAttributeViolation;

const DC_BOUNDS_REQUIRED_ATTRIBUTES: &[&str] = &["x", "y", "width", "height"];
const DI_WAYPOINT_REQUIRED_ATTRIBUTES: &[&str] = &["x", "y"];

pub(super) fn collect_required_attribute_violations(
    source: &BpmnSourceFile,
) -> Option<Vec<DiRequiredAttributeViolation>> {
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
    violations: &mut Vec<DiRequiredAttributeViolation>,
) {
    let event_name = event.name();
    let Some(element) = local_name(event_name.as_ref()) else {
        return;
    };
    let Some(required_attributes) = required_attributes_for_element(element) else {
        return;
    };

    let event_path = event_path(parent_path, element);
    for attribute_name in required_attributes {
        if has_attribute(event, attribute_name) {
            continue;
        }
        violations.push(DiRequiredAttributeViolation::new(
            element,
            attribute_value(reader, event, "id"),
            &event_path,
            attribute_name,
            required_attributes,
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

fn has_attribute(event: &BytesStart<'_>, name: &str) -> bool {
    event
        .attributes()
        .flatten()
        .any(|attribute| local_name(attribute.key.as_ref()) == Some(name))
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

fn required_attributes_for_element(element: &str) -> Option<&'static [&'static str]> {
    match element {
        "Bounds" => Some(DC_BOUNDS_REQUIRED_ATTRIBUTES),
        "waypoint" => Some(DI_WAYPOINT_REQUIRED_ATTRIBUTES),
        _ => None,
    }
}
