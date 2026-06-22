use xiuxian_qianji_bpmn_engine::{DmnLabelSnapshot, DmnShapeSnapshot, DmnWaypointSnapshot};

pub(super) fn assert_shape_bounds(
    shape: &DmnShapeSnapshot,
    x: Option<&str>,
    y: Option<&str>,
    width: Option<&str>,
    height: Option<&str>,
) {
    assert_eq!(
        shape.bounds.as_ref().and_then(|bounds| bounds.x.as_deref()),
        x
    );
    assert_eq!(
        shape.bounds.as_ref().and_then(|bounds| bounds.y.as_deref()),
        y
    );
    assert_eq!(
        shape
            .bounds
            .as_ref()
            .and_then(|bounds| bounds.width.as_deref()),
        width
    );
    assert_eq!(
        shape
            .bounds
            .as_ref()
            .and_then(|bounds| bounds.height.as_deref()),
        height
    );
}

pub(super) fn assert_label_bounds(
    label: Option<&DmnLabelSnapshot>,
    x: Option<&str>,
    y: Option<&str>,
    width: Option<&str>,
    height: Option<&str>,
) {
    let bounds = label.and_then(|label| label.bounds.as_ref());
    assert_eq!(bounds.and_then(|bounds| bounds.x.as_deref()), x);
    assert_eq!(bounds.and_then(|bounds| bounds.y.as_deref()), y);
    assert_eq!(bounds.and_then(|bounds| bounds.width.as_deref()), width);
    assert_eq!(bounds.and_then(|bounds| bounds.height.as_deref()), height);
}

pub(super) fn assert_waypoint(waypoint: &DmnWaypointSnapshot, x: Option<&str>, y: Option<&str>) {
    assert_eq!(waypoint.x.as_deref(), x);
    assert_eq!(waypoint.y.as_deref(), y);
}
