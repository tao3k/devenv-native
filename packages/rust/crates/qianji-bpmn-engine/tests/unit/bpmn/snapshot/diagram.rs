use super::snapshot_fixture;
use qianji_bpmn_engine::{BpmnBoundsSnapshot, BpmnWaypointSnapshot};

#[test]
fn bpmn_snapshot_preserves_diagram_metadata() {
    let snapshot = snapshot_fixture("metadata-bpmn-diagram.bpmn");

    assert_eq!(snapshot.root.diagram_count, 1);
    assert_eq!(snapshot.root.diagrams.len(), 1);
    let diagram = &snapshot.root.diagrams[0];
    assert_eq!(diagram.diagram_id.as_deref(), Some("Diagram_Main"));
    assert_eq!(diagram.name.as_deref(), Some("Main diagram"));
    assert_eq!(
        diagram.documentation.as_deref(),
        Some("Passive interchange layout")
    );
    assert_eq!(diagram.resolution.as_deref(), Some("96"));
    assert_eq!(diagram.label_styles.len(), 1);
    let style = &diagram.label_styles[0];
    assert_eq!(style.style_id.as_deref(), Some("Style_Default"));
    let Some(font) = style.font.as_ref() else {
        panic!("label style should carry font");
    };
    assert_eq!(font.name.as_deref(), Some("Inter"));
    assert_eq!(font.size.as_deref(), Some("12"));
    assert_eq!(font.is_bold, Some(true));
    assert_eq!(font.is_italic, Some(false));
    assert_eq!(font.is_underline, Some(false));
    assert_eq!(font.is_strike_through, Some(false));

    let Some(plane) = diagram.plane.as_ref() else {
        panic!("diagram should carry a plane");
    };
    assert_eq!(plane.plane_id.as_deref(), Some("Plane_Main"));
    assert_eq!(plane.bpmn_element.as_deref(), Some("diagram_process"));
    assert_eq!(plane.shapes.len(), 1);
    assert_eq!(plane.edges.len(), 1);

    let shape = &plane.shapes[0];
    assert_eq!(shape.shape_id.as_deref(), Some("Shape_Start"));
    assert_eq!(shape.bpmn_element.as_deref(), Some("start"));
    assert_eq!(shape.is_horizontal, Some(true));
    assert_eq!(shape.is_expanded, Some(false));
    assert_eq!(shape.is_marker_visible, Some(true));
    assert_eq!(shape.is_message_visible, None);
    assert_bounds(
        shape.bounds.as_ref(),
        Some("100"),
        Some("80"),
        Some("36"),
        Some("36"),
    );
    let Some(shape_label) = shape.label.as_ref() else {
        panic!("shape should carry a BPMN label");
    };
    assert_eq!(shape_label.label_id.as_deref(), Some("Label_Start"));
    assert_eq!(shape_label.label_style.as_deref(), Some("Style_Default"));
    assert_bounds(
        shape_label.bounds.as_ref(),
        Some("92"),
        Some("120"),
        Some("52"),
        Some("18"),
    );

    let edge = &plane.edges[0];
    assert_eq!(edge.edge_id.as_deref(), Some("Edge_Start_End"));
    assert_eq!(edge.bpmn_element.as_deref(), Some("flow_start_end"));
    assert_eq!(edge.source_element.as_deref(), Some("Shape_Start"));
    assert_eq!(edge.target_element.as_deref(), Some("Shape_End"));
    assert_eq!(edge.message_visible_kind.as_deref(), Some("initiating"));
    assert_eq!(edge.waypoints.len(), 2);
    assert_waypoint(&edge.waypoints[0], Some("136"), Some("98"));
    assert_waypoint(&edge.waypoints[1], Some("220"), Some("98"));
    let Some(edge_label) = edge.label.as_ref() else {
        panic!("edge should carry a BPMN label");
    };
    assert_eq!(edge_label.label_id.as_deref(), Some("Label_Flow"));
    assert_bounds(
        edge_label.bounds.as_ref(),
        Some("160"),
        Some("78"),
        Some("36"),
        Some("18"),
    );
}

fn assert_bounds(
    bounds: Option<&BpmnBoundsSnapshot>,
    x: Option<&str>,
    y: Option<&str>,
    width: Option<&str>,
    height: Option<&str>,
) {
    let Some(bounds) = bounds else {
        panic!("bounds should be preserved");
    };
    assert_eq!(bounds.x.as_deref(), x);
    assert_eq!(bounds.y.as_deref(), y);
    assert_eq!(bounds.width.as_deref(), width);
    assert_eq!(bounds.height.as_deref(), height);
}

fn assert_waypoint(waypoint: &BpmnWaypointSnapshot, x: Option<&str>, y: Option<&str>) {
    assert_eq!(waypoint.x.as_deref(), x);
    assert_eq!(waypoint.y.as_deref(), y);
}
