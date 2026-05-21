use crate::{LaneCapability, PolyglotLane};

#[test]
fn lane_identifiers_are_stable() {
    assert_eq!(PolyglotLane::PythonDocling.as_str(), "python_docling");
    assert_eq!(PolyglotLane::JuliaCompute.as_str(), "julia_compute");
}

#[test]
fn capabilities_map_to_existing_lane_owners() {
    assert_eq!(
        LaneCapability::DocumentExtraction.owning_lane(),
        PolyglotLane::PythonDocling
    );
    assert_eq!(
        LaneCapability::OcrShardExtraction.owning_lane(),
        PolyglotLane::PythonDocling
    );
    assert_eq!(
        LaneCapability::GraphEvidenceCompute.owning_lane(),
        PolyglotLane::JuliaCompute
    );
    assert_eq!(
        LaneCapability::GraphSearchCompute.owning_lane(),
        PolyglotLane::JuliaCompute
    );
    assert_eq!(
        LaneCapability::ScientificCompute.owning_lane(),
        PolyglotLane::JuliaCompute
    );
    assert_eq!(
        LaneCapability::MemoryProfileCompute.owning_lane(),
        PolyglotLane::JuliaCompute
    );
}

#[test]
fn lane_serialization_is_snake_case() -> Result<(), serde_json::Error> {
    let serialized = serde_json::to_string(&PolyglotLane::PythonDocling)?;
    assert_eq!(serialized, "\"python_docling\"");
    Ok(())
}
