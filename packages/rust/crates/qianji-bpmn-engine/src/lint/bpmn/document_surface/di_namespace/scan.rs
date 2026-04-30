use std::collections::BTreeMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::xml::local_name;

use super::model::DiNamespaceViolation;

const BPMN_DI_NAMESPACE: &str = "http://www.omg.org/spec/BPMN/20100524/DI";
const DC_NAMESPACE: &str = "http://www.omg.org/spec/DD/20100524/DC";
const DI_NAMESPACE: &str = "http://www.omg.org/spec/DD/20100524/DI";

type NamespaceScope = BTreeMap<String, String>;

pub(super) fn collect_namespace_violations(
    source: &BpmnSourceFile,
) -> Option<Vec<DiNamespaceViolation>> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);
    let mut current_scope = NamespaceScope::new();
    let mut scope_stack = Vec::<NamespaceScope>::new();
    let mut path = Vec::<String>::new();
    let mut violations = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let next_scope = scoped_namespace(&reader, &event, &current_scope);
                inspect_event(&reader, &event, &next_scope, &path, &mut violations);
                scope_stack.push(current_scope);
                current_scope = next_scope;
                push_path(&event, &mut path);
            }
            Ok(Event::Empty(event)) => {
                let next_scope = scoped_namespace(&reader, &event, &current_scope);
                inspect_event(&reader, &event, &next_scope, &path, &mut violations);
            }
            Ok(Event::End(_)) => {
                let _ = path.pop();
                current_scope = scope_stack.pop().unwrap_or_default();
            }
            Ok(Event::Eof) => return Some(violations),
            Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn scoped_namespace(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    current_scope: &NamespaceScope,
) -> NamespaceScope {
    let mut next_scope = current_scope.clone();
    for attribute in event.attributes().flatten() {
        let Some(name) = std::str::from_utf8(attribute.key.as_ref()).ok() else {
            continue;
        };
        let prefix = if name == "xmlns" {
            Some("")
        } else {
            name.strip_prefix("xmlns:")
        };
        let Some(prefix) = prefix else {
            continue;
        };
        if let Ok(value) = attribute.decode_and_unescape_value(reader.decoder()) {
            next_scope.insert(prefix.to_string(), value.into_owned());
        }
    }
    next_scope
}

fn inspect_event(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    scope: &NamespaceScope,
    parent_path: &[String],
    violations: &mut Vec<DiNamespaceViolation>,
) {
    let event_name = event.name();
    let Some(element) = local_name(event_name.as_ref()) else {
        return;
    };
    let Some(expected_namespace_uri) = expected_namespace_for_element(element) else {
        return;
    };
    let prefix = element_prefix(event_name.as_ref());
    let namespace_uri = scope.get(prefix.unwrap_or("")).map(String::as_str);
    if namespace_uri == Some(expected_namespace_uri) {
        return;
    }
    let element_id = attribute_value(reader, event, "id");
    violations.push(DiNamespaceViolation::new(
        element,
        element_id,
        &event_path(parent_path, element),
        prefix,
        namespace_uri,
        expected_namespace_uri,
    ));
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

fn element_prefix(raw_name: &[u8]) -> Option<&str> {
    let name = std::str::from_utf8(raw_name).ok()?;
    name.rsplit_once(':').map(|(prefix, _)| prefix)
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

fn expected_namespace_for_element(element: &str) -> Option<&'static str> {
    match element {
        "BPMNDiagram" | "BPMNPlane" | "BPMNShape" | "BPMNEdge" | "BPMNLabel" | "BPMNLabelStyle" => {
            Some(BPMN_DI_NAMESPACE)
        }
        "Bounds" | "Font" => Some(DC_NAMESPACE),
        "waypoint" => Some(DI_NAMESPACE),
        _ => None,
    }
}
