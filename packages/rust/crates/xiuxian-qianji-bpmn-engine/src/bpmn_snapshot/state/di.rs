use super::{
    BpmnDiLabelTarget, BpmnDiagramSnapshot, BpmnEdgeSnapshot, BpmnLabelSnapshot,
    BpmnLabelStyleSnapshot, BpmnPlaneSnapshot, BpmnShapeSnapshot, BpmnSnapshotScanState,
    BpmnSourceFile, BytesStart, Reader, Result, attribute_value, boolean_attribute_value,
    bounds_from_event, font_from_event, label_from_event, waypoint_from_event,
};

impl BpmnSnapshotScanState {
    pub(super) fn start_bpmn_diagram(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(root) = self.root.as_mut() else {
            return Ok(());
        };
        root.diagram_count += 1;
        root.diagrams.push(BpmnDiagramSnapshot {
            diagram_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            documentation: attribute_value(source, reader, event, "documentation")?,
            resolution: attribute_value(source, reader, event, "resolution")?,
            plane: None,
            label_styles: Vec::new(),
        });
        if !is_empty {
            self.current_diagram = root.diagrams.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_bpmn_plane(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_diagram else {
            return Ok(());
        };
        let Some(diagram) = self.diagram_mut(diagram_index) else {
            return Ok(());
        };
        diagram.plane = Some(BpmnPlaneSnapshot {
            plane_id: attribute_value(source, reader, event, "id")?,
            bpmn_element: attribute_value(source, reader, event, "bpmnElement")?,
            shapes: Vec::new(),
            edges: Vec::new(),
        });
        if !is_empty {
            self.current_plane = Some(diagram_index);
        }
        Ok(())
    }

    pub(super) fn start_bpmn_shape(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_plane else {
            return Ok(());
        };
        let shape = BpmnShapeSnapshot {
            shape_id: attribute_value(source, reader, event, "id")?.map(Into::into),
            bpmn_element: attribute_value(source, reader, event, "bpmnElement")?,
            is_horizontal: boolean_attribute_value(source, reader, event, "isHorizontal")?
                .map(Into::into),
            is_expanded: boolean_attribute_value(source, reader, event, "isExpanded")?
                .map(Into::into),
            is_marker_visible: boolean_attribute_value(source, reader, event, "isMarkerVisible")?
                .map(Into::into),
            is_message_visible: boolean_attribute_value(source, reader, event, "isMessageVisible")?
                .map(Into::into),
            participant_band_kind: attribute_value(source, reader, event, "participantBandKind")?
                .map(Into::into),
            choreography_activity_shape: attribute_value(
                source,
                reader,
                event,
                "choreographyActivityShape",
            )?,
            bounds: None,
            label: None,
        };
        let Some(plane) = self.diagram_plane_mut(diagram_index) else {
            return Ok(());
        };
        plane.shapes.push(shape);
        let shape_index = plane.shapes.len().saturating_sub(1);
        if !is_empty {
            self.current_shape = Some((diagram_index, shape_index));
        }
        Ok(())
    }

    pub(super) fn start_bpmn_edge(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_plane else {
            return Ok(());
        };
        let edge = BpmnEdgeSnapshot {
            edge_id: attribute_value(source, reader, event, "id")?,
            bpmn_element: attribute_value(source, reader, event, "bpmnElement")?,
            source_element: attribute_value(source, reader, event, "sourceElement")?,
            target_element: attribute_value(source, reader, event, "targetElement")?,
            message_visible_kind: attribute_value(source, reader, event, "messageVisibleKind")?
                .map(Into::into),
            waypoints: Vec::new(),
            label: None,
        };
        let Some(plane) = self.diagram_plane_mut(diagram_index) else {
            return Ok(());
        };
        plane.edges.push(edge);
        let edge_index = plane.edges.len().saturating_sub(1);
        if !is_empty {
            self.current_edge = Some((diagram_index, edge_index));
        }
        Ok(())
    }

    pub(super) fn start_bpmn_shape_label(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((diagram_index, shape_index)) = self.current_shape else {
            return Ok(());
        };
        let Some(shape) = self.diagram_shape_mut(diagram_index, shape_index) else {
            return Ok(());
        };
        shape.label = Some(label_from_event(source, reader, event)?);
        if !is_empty {
            self.current_label = Some(BpmnDiLabelTarget::Shape(diagram_index, shape_index));
        }
        Ok(())
    }

    pub(super) fn start_bpmn_edge_label(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some((diagram_index, edge_index)) = self.current_edge else {
            return Ok(());
        };
        let Some(edge) = self.diagram_edge_mut(diagram_index, edge_index) else {
            return Ok(());
        };
        edge.label = Some(label_from_event(source, reader, event)?);
        if !is_empty {
            self.current_label = Some(BpmnDiLabelTarget::Edge(diagram_index, edge_index));
        }
        Ok(())
    }

    pub(super) fn start_bpmn_label_style(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(diagram_index) = self.current_diagram else {
            return Ok(());
        };
        let Some(diagram) = self.diagram_mut(diagram_index) else {
            return Ok(());
        };
        diagram.label_styles.push(BpmnLabelStyleSnapshot {
            style_id: attribute_value(source, reader, event, "id")?,
            font: None,
        });
        let style_index = diagram.label_styles.len().saturating_sub(1);
        if !is_empty {
            self.current_label_style = Some((diagram_index, style_index));
        }
        Ok(())
    }

    pub(super) fn attach_bpmn_shape_bounds(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((diagram_index, shape_index)) = self.current_shape else {
            return Ok(());
        };
        let Some(shape) = self.diagram_shape_mut(diagram_index, shape_index) else {
            return Ok(());
        };
        shape.bounds = Some(bounds_from_event(source, reader, event)?);
        Ok(())
    }

    pub(super) fn attach_bpmn_label_bounds(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(target) = self.current_label else {
            return Ok(());
        };
        let Some(label) = self.diagram_label_mut(target) else {
            return Ok(());
        };
        label.bounds = Some(bounds_from_event(source, reader, event)?);
        Ok(())
    }

    pub(super) fn push_bpmn_edge_waypoint(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((diagram_index, edge_index)) = self.current_edge else {
            return Ok(());
        };
        let Some(edge) = self.diagram_edge_mut(diagram_index, edge_index) else {
            return Ok(());
        };
        edge.waypoints
            .push(waypoint_from_event(source, reader, event)?);
        Ok(())
    }

    pub(super) fn attach_bpmn_label_style_font(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some((diagram_index, style_index)) = self.current_label_style else {
            return Ok(());
        };
        let Some(style) = self.diagram_label_style_mut(diagram_index, style_index) else {
            return Ok(());
        };
        style.font = Some(font_from_event(source, reader, event)?);
        Ok(())
    }

    pub(super) fn diagram_mut(&mut self, diagram_index: usize) -> Option<&mut BpmnDiagramSnapshot> {
        self.root.as_mut()?.diagrams.get_mut(diagram_index)
    }

    pub(super) fn diagram_plane_mut(
        &mut self,
        diagram_index: usize,
    ) -> Option<&mut BpmnPlaneSnapshot> {
        self.diagram_mut(diagram_index)?.plane.as_mut()
    }

    pub(super) fn diagram_shape_mut(
        &mut self,
        diagram_index: usize,
        shape_index: usize,
    ) -> Option<&mut BpmnShapeSnapshot> {
        self.diagram_plane_mut(diagram_index)?
            .shapes
            .get_mut(shape_index)
    }

    pub(super) fn diagram_edge_mut(
        &mut self,
        diagram_index: usize,
        edge_index: usize,
    ) -> Option<&mut BpmnEdgeSnapshot> {
        self.diagram_plane_mut(diagram_index)?
            .edges
            .get_mut(edge_index)
    }

    pub(super) fn diagram_label_mut(
        &mut self,
        target: BpmnDiLabelTarget,
    ) -> Option<&mut BpmnLabelSnapshot> {
        match target {
            BpmnDiLabelTarget::Shape(diagram_index, shape_index) => self
                .diagram_shape_mut(diagram_index, shape_index)?
                .label
                .as_mut(),
            BpmnDiLabelTarget::Edge(diagram_index, edge_index) => self
                .diagram_edge_mut(diagram_index, edge_index)?
                .label
                .as_mut(),
        }
    }

    pub(super) fn diagram_label_style_mut(
        &mut self,
        diagram_index: usize,
        style_index: usize,
    ) -> Option<&mut BpmnLabelStyleSnapshot> {
        self.diagram_mut(diagram_index)?
            .label_styles
            .get_mut(style_index)
    }
}
