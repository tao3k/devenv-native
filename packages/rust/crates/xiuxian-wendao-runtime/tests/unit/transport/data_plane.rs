use super::{
    WENDAO_ARROW_FLIGHT_DATA_PLANE, WENDAO_ARROW_RECORD_BATCH_PAYLOAD,
    WENDAO_FORBIDDEN_ARROW_PAYLOAD_WRAPPERS, WENDAO_JSONL_STDIO_CONTROL_PLANE,
    WENDAO_PROCESS_ARGS_CONTROL_PLANE, WENDAO_REST_METADATA_CONTROL_PLANE, WendaoPayloadPlane,
    WendaoPayloadSurface, parse_wendao_payload_surface, validate_arrow_payload_encoding_token,
    validate_arrow_table_payload_surface,
};

#[test]
fn arrow_flight_and_record_batch_are_data_plane_surfaces() {
    assert_eq!(
        parse_wendao_payload_surface(WENDAO_ARROW_FLIGHT_DATA_PLANE),
        Some(WendaoPayloadSurface::ArrowFlight)
    );
    assert_eq!(
        parse_wendao_payload_surface(WENDAO_ARROW_RECORD_BATCH_PAYLOAD),
        Some(WendaoPayloadSurface::ArrowRecordBatch)
    );
    assert_eq!(
        WendaoPayloadSurface::ArrowFlight.plane(),
        WendaoPayloadPlane::Data
    );
    assert_eq!(
        WendaoPayloadSurface::ArrowRecordBatch.plane(),
        WendaoPayloadPlane::Data
    );
    assert!(validate_arrow_table_payload_surface(WendaoPayloadSurface::ArrowFlight).is_ok());
    assert!(validate_arrow_table_payload_surface(WendaoPayloadSurface::ArrowRecordBatch).is_ok());
}

#[test]
fn jsonl_process_args_and_rest_metadata_are_control_only() {
    assert_eq!(
        parse_wendao_payload_surface(WENDAO_JSONL_STDIO_CONTROL_PLANE),
        Some(WendaoPayloadSurface::JsonlStdioControl)
    );
    assert_eq!(
        parse_wendao_payload_surface(WENDAO_PROCESS_ARGS_CONTROL_PLANE),
        Some(WendaoPayloadSurface::ProcessArgsControl)
    );
    assert_eq!(
        parse_wendao_payload_surface(WENDAO_REST_METADATA_CONTROL_PLANE),
        Some(WendaoPayloadSurface::RestMetadataControl)
    );
    assert_eq!(
        WendaoPayloadSurface::JsonlStdioControl.plane(),
        WendaoPayloadPlane::Control
    );
    assert_eq!(
        WendaoPayloadSurface::ProcessArgsControl.plane(),
        WendaoPayloadPlane::Control
    );
    assert_eq!(
        WendaoPayloadSurface::RestMetadataControl.plane(),
        WendaoPayloadPlane::Control
    );
}

#[test]
fn control_surfaces_reject_arrow_table_payloads() {
    for surface in [
        WendaoPayloadSurface::JsonControl,
        WendaoPayloadSurface::JsonlStdioControl,
        WendaoPayloadSurface::ProcessArgsControl,
        WendaoPayloadSurface::RestMetadataControl,
    ] {
        let error = match validate_arrow_table_payload_surface(surface) {
            Ok(()) => panic!("control surfaces must not carry Arrow table payloads"),
            Err(error) => error,
        };
        assert!(
            error.message().contains("control-only"),
            "expected control-only error, got: {error}"
        );
        assert!(
            error.message().contains(WENDAO_ARROW_FLIGHT_DATA_PLANE),
            "expected canonical data-plane token, got: {error}"
        );
    }
}

#[test]
fn base64_arrow_ipc_wrappers_are_rejected() {
    let legacy_trace_wrapper = ["trace", "Arrow", "Ipc", "Base64"].join("");
    for token in WENDAO_FORBIDDEN_ARROW_PAYLOAD_WRAPPERS
        .iter()
        .copied()
        .chain(std::iter::once(legacy_trace_wrapper.as_str()))
    {
        let error = match validate_arrow_payload_encoding_token(token) {
            Ok(()) => panic!("Arrow/base64 wrappers must be rejected"),
            Err(error) => error,
        };
        assert!(
            error.message().contains(WENDAO_ARROW_FLIGHT_DATA_PLANE),
            "expected Arrow Flight guidance, got: {error}"
        );
    }

    assert!(validate_arrow_payload_encoding_token(WENDAO_ARROW_FLIGHT_DATA_PLANE).is_ok());
    assert!(validate_arrow_payload_encoding_token(WENDAO_JSONL_STDIO_CONTROL_PLANE).is_ok());
}
