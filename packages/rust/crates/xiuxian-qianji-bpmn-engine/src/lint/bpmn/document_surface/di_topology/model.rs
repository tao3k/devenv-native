use serde_json::{Value, json};

#[derive(Debug, Default)]
pub(super) struct DiTopologyScan {
    diagrams: Vec<DiTopologyDiagram>,
    orphan_planes: Vec<DiTopologyViolation>,
}

#[derive(Debug)]
pub(super) struct DiTopologyDiagram {
    diagram_id: Option<String>,
    plane_ids: Vec<Option<String>>,
}

#[derive(Debug)]
pub(super) struct DiTopologyViolation {
    diagram_id: Option<String>,
    plane_id: Option<String>,
    element: &'static str,
    reason: &'static str,
    expected: &'static str,
    observed_count: usize,
    observed_plane_ids: Vec<Option<String>>,
    parent: Option<String>,
}

impl DiTopologyScan {
    pub(super) fn push_diagram(&mut self, diagram_id: Option<String>) -> usize {
        self.diagrams.push(DiTopologyDiagram {
            diagram_id,
            plane_ids: Vec::new(),
        });
        self.diagrams.len().saturating_sub(1)
    }

    pub(super) fn push_plane(&mut self, diagram_index: usize, plane_id: Option<String>) {
        if let Some(diagram) = self.diagrams.get_mut(diagram_index) {
            diagram.plane_ids.push(plane_id);
        }
    }

    pub(super) fn push_orphan_plane(&mut self, plane_id: Option<String>, parent: Option<&str>) {
        self.orphan_planes
            .push(DiTopologyViolation::orphan_plane(plane_id, parent));
    }

    pub(super) fn violations(self) -> Vec<DiTopologyViolation> {
        let mut violations = Vec::new();
        for diagram in self.diagrams {
            if diagram.plane_ids.is_empty() {
                violations.push(DiTopologyViolation::missing_plane(diagram.diagram_id));
                continue;
            }
            if diagram.plane_ids.len() > 1 {
                violations.push(DiTopologyViolation::multiple_planes(
                    diagram.diagram_id,
                    diagram.plane_ids,
                ));
            }
        }
        violations.extend(self.orphan_planes);
        violations
    }
}

impl DiTopologyViolation {
    fn missing_plane(diagram_id: Option<String>) -> Self {
        Self {
            diagram_id,
            plane_id: None,
            element: "BPMNDiagram",
            reason: "missing_direct_plane",
            expected: "exactly_one_direct_BPMNPlane",
            observed_count: 0,
            observed_plane_ids: Vec::new(),
            parent: None,
        }
    }

    fn multiple_planes(
        diagram_id: Option<String>,
        observed_plane_ids: Vec<Option<String>>,
    ) -> Self {
        Self {
            diagram_id,
            plane_id: None,
            element: "BPMNDiagram",
            reason: "multiple_direct_planes",
            expected: "exactly_one_direct_BPMNPlane",
            observed_count: observed_plane_ids.len(),
            observed_plane_ids,
            parent: None,
        }
    }

    fn orphan_plane(plane_id: Option<String>, parent: Option<&str>) -> Self {
        Self {
            diagram_id: None,
            plane_id,
            element: "BPMNPlane",
            reason: "plane_outside_diagram",
            expected: "direct_child_of_BPMNDiagram",
            observed_count: 1,
            observed_plane_ids: Vec::new(),
            parent: parent.map(str::to_string),
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "diagram_id": self.diagram_id.as_deref(),
            "plane_id": self.plane_id.as_deref(),
            "element": self.element,
            "reason": self.reason,
            "expected": self.expected,
            "observed_count": self.observed_count,
            "observed_plane_ids": self.observed_plane_ids,
            "parent": self.parent.as_deref(),
        })
    }
}
