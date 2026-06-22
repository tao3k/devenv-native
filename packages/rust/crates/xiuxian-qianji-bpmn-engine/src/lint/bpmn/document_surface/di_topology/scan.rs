use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::bpmn_parse_api::BpmnSourceFile;

use super::model::{DiTopologyScan, DiTopologyViolation};
use super::xml::{attribute_value, local_name};

pub(super) fn collect_topology_violations(source: &BpmnSourceFile) -> Vec<DiTopologyViolation> {
    let scanner = DiTopologyScanner::default();
    scanner.scan(source)
}

#[derive(Default)]
struct DiTopologyScanner {
    stack: Vec<String>,
    active_diagrams: Vec<usize>,
    topology: DiTopologyScan,
}

impl DiTopologyScanner {
    fn scan(mut self, source: &BpmnSourceFile) -> Vec<DiTopologyViolation> {
        let mut reader = Reader::from_str(&source.contents);
        reader.config_mut().trim_text(true);

        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => {
                    self.handle_start(&reader, &event, false);
                }
                Ok(Event::Empty(event)) => {
                    self.handle_start(&reader, &event, true);
                }
                Ok(Event::End(event)) => {
                    if local_name(event.name().as_ref()) == Some("BPMNDiagram") {
                        let _ = self.active_diagrams.pop();
                    }
                    let _ = self.stack.pop();
                }
                Ok(Event::Eof) | Err(_) => break,
                Ok(_) => {}
            }
        }

        self.topology.violations()
    }

    fn handle_start(&mut self, reader: &Reader<&[u8]>, event: &BytesStart<'_>, is_empty: bool) {
        let name = event.name();
        let Some(tag) = local_name(name.as_ref()) else {
            return;
        };
        let parent = self.stack.last().map(String::as_str);

        match tag {
            "BPMNDiagram" => {
                let diagram_id = attribute_value(reader, event, "id");
                let diagram_index = self.topology.push_diagram(diagram_id);
                if !is_empty {
                    self.active_diagrams.push(diagram_index);
                }
            }
            "BPMNPlane" => {
                let plane_id = attribute_value(reader, event, "id");
                if parent == Some("BPMNDiagram")
                    && let Some(diagram_index) = self.active_diagrams.last().copied()
                {
                    self.topology.push_plane(diagram_index, plane_id);
                } else {
                    self.topology.push_orphan_plane(plane_id, parent);
                }
            }
            _ => {}
        }

        if !is_empty {
            self.stack.push(tag.to_string());
        }
    }
}
