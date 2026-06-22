use super::placeholders::{
    append_label_text, bounds_from_event, label_from_event, waypoint_from_event,
};
use crate::dmn::snapshot::xml::{attribute_value, boolean_attribute_value};
use crate::dmn_model_api::{
    DmnDecisionServiceDividerLineSnapshot, DmnDiagramSnapshot, DmnDmndiSnapshot, DmnEdgeSnapshot,
    DmnShapeSnapshot, DmnSourceFile,
};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(in crate::dmn::snapshot::state) struct TempDmndiSnapshot {
    dmndi_id: Option<String>,
    diagrams: Vec<DmnDiagramSnapshot>,
}

impl From<TempDmndiSnapshot> for DmnDmndiSnapshot {
    fn from(value: TempDmndiSnapshot) -> Self {
        Self {
            dmndi_id: value.dmndi_id,
            diagrams: value.diagrams,
        }
    }
}

impl TempDmndiSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            dmndi_id: attribute_value(source, reader, event, "id")?,
            diagrams: Vec::new(),
        })
    }

    pub(in crate::dmn::snapshot::state) fn push_diagram(&mut self, diagram: DmnDiagramSnapshot) {
        self.diagrams.push(diagram);
    }
}

#[derive(Debug)]
pub(in crate::dmn::snapshot::state) struct TempDiagramSnapshot {
    diagram_id: Option<String>,
    shapes: Vec<DmnShapeSnapshot>,
    edges: Vec<DmnEdgeSnapshot>,
}

impl From<TempDiagramSnapshot> for DmnDiagramSnapshot {
    fn from(value: TempDiagramSnapshot) -> Self {
        Self {
            diagram_id: value.diagram_id,
            shape_count: value.shapes.len(),
            edge_count: value.edges.len(),
            shapes: value.shapes,
            edges: value.edges,
        }
    }
}

impl TempDiagramSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            diagram_id: attribute_value(source, reader, event, "id")?,
            shapes: Vec::new(),
            edges: Vec::new(),
        })
    }

    pub(in crate::dmn::snapshot::state) fn push_shape_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        self.shapes.push(DmnShapeSnapshot {
            shape_id: attribute_value(source, reader, event, "id")?.map(Into::into),
            dmn_element_ref: attribute_value(source, reader, event, "dmnElementRef")?,
            is_listed_input_data: boolean_attribute_value(
                source,
                reader,
                event,
                "isListedInputData",
            )?
            .map(Into::into),
            is_collapsed: boolean_attribute_value(source, reader, event, "isCollapsed")?
                .map(Into::into),
            bounds: None,
            decision_service_divider_line: None,
            label: None,
        });
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn attach_shape_decision_service_divider_line(&mut self) {
        let Some(shape) = self.shapes.last_mut() else {
            return;
        };
        shape.decision_service_divider_line = Some(DmnDecisionServiceDividerLineSnapshot {
            waypoints: Vec::new(),
        });
    }

    pub(in crate::dmn::snapshot::state) fn push_shape_decision_service_divider_line_waypoint(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(divider_line) = self
            .shapes
            .last_mut()
            .and_then(|shape| shape.decision_service_divider_line.as_mut())
        else {
            return Ok(());
        };
        divider_line
            .waypoints
            .push(waypoint_from_event(source, reader, event)?);
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn push_edge_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        self.edges.push(DmnEdgeSnapshot {
            edge_id: attribute_value(source, reader, event, "id")?,
            dmn_element_ref: attribute_value(source, reader, event, "dmnElementRef")?,
            waypoints: Vec::new(),
            label: None,
        });
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn push_edge_waypoint_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(edge) = self.edges.last_mut() else {
            return Ok(());
        };
        edge.waypoints
            .push(waypoint_from_event(source, reader, event)?);
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn attach_shape_label_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(shape) = self.shapes.last_mut() else {
            return Ok(());
        };
        shape.label = Some(label_from_event(source, reader, event)?);
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn attach_shape_label_bounds_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(label) = self
            .shapes
            .last_mut()
            .and_then(|shape| shape.label.as_mut())
        else {
            return Ok(());
        };
        label.bounds = Some(bounds_from_event(source, reader, event)?);
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn attach_shape_bounds_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(shape) = self.shapes.last_mut() else {
            return Ok(());
        };
        shape.bounds = Some(bounds_from_event(source, reader, event)?);
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn attach_edge_label_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(edge) = self.edges.last_mut() else {
            return Ok(());
        };
        edge.label = Some(label_from_event(source, reader, event)?);
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn attach_edge_label_bounds_from_event(
        &mut self,
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(label) = self.edges.last_mut().and_then(|edge| edge.label.as_mut()) else {
            return Ok(());
        };
        label.bounds = Some(bounds_from_event(source, reader, event)?);
        Ok(())
    }

    pub(in crate::dmn::snapshot::state) fn append_shape_label_text(&mut self, text: &str) {
        let Some(label) = self
            .shapes
            .last_mut()
            .and_then(|shape| shape.label.as_mut())
        else {
            return;
        };
        append_label_text(label, text);
    }

    pub(in crate::dmn::snapshot::state) fn append_edge_label_text(&mut self, text: &str) {
        let Some(label) = self.edges.last_mut().and_then(|edge| edge.label.as_mut()) else {
            return;
        };
        append_label_text(label, text);
    }
}
