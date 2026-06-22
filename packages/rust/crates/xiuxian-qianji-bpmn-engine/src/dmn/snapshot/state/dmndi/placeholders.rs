use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{
    DmnBoundsSnapshot, DmnLabelSnapshot, DmnSourceFile, DmnWaypointSnapshot,
};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(super) fn label_from_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<DmnLabelSnapshot> {
    Ok(DmnLabelSnapshot {
        label_id: attribute_value(source, reader, event, "id")?,
        bounds: None,
        text: None,
    })
}

pub(super) fn bounds_from_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<DmnBoundsSnapshot> {
    Ok(DmnBoundsSnapshot {
        x: attribute_value(source, reader, event, "x")?,
        y: attribute_value(source, reader, event, "y")?,
        width: attribute_value(source, reader, event, "width")?,
        height: attribute_value(source, reader, event, "height")?,
    })
}

pub(super) fn waypoint_from_event(
    source: &DmnSourceFile,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<DmnWaypointSnapshot> {
    Ok(DmnWaypointSnapshot {
        x: attribute_value(source, reader, event, "x")?,
        y: attribute_value(source, reader, event, "y")?,
    })
}

pub(super) fn append_label_text(label: &mut DmnLabelSnapshot, text: &str) {
    match label.text.as_mut() {
        Some(existing) => existing.push_str(text),
        None => label.text = Some(text.to_string()),
    }
}
