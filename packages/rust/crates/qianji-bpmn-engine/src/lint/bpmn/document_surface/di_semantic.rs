use std::collections::BTreeMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::xml::local_name;

#[derive(Debug)]
pub(super) struct SemanticElementIndex {
    elements: BTreeMap<String, String>,
}

impl SemanticElementIndex {
    pub(super) fn from_source(source: &BpmnSourceFile) -> Option<Self> {
        let mut reader = Reader::from_str(&source.contents);
        reader.config_mut().trim_text(true);
        let mut elements = BTreeMap::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(event) | Event::Empty(event)) => {
                    collect_semantic_element(&reader, &event, &mut elements);
                }
                Ok(Event::Eof) => return Some(Self { elements }),
                Err(_) => return None,
                Ok(_) => {}
            }
        }
    }

    pub(super) fn contains_id(&self, id: &str) -> bool {
        self.elements.contains_key(id)
    }

    pub(super) fn len(&self) -> usize {
        self.elements.len()
    }

    pub(super) fn tag_for(&self, id: &str) -> Option<&str> {
        self.elements.get(id).map(String::as_str)
    }
}

fn collect_semantic_element(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    elements: &mut BTreeMap<String, String>,
) {
    let event_name = event.name();
    let Some(tag) = local_name(event_name.as_ref()) else {
        return;
    };
    if is_di_metadata_tag(tag) {
        return;
    }
    let Some(id) = attribute_value(reader, event, "id") else {
        return;
    };
    elements.entry(id).or_insert_with(|| tag.to_string());
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

fn is_di_metadata_tag(tag: &str) -> bool {
    matches!(
        tag,
        "BPMNDiagram"
            | "BPMNPlane"
            | "BPMNShape"
            | "BPMNEdge"
            | "BPMNLabel"
            | "BPMNLabelStyle"
            | "Bounds"
            | "waypoint"
            | "Font"
    )
}
