use std::sync::Arc;

use arrow::array::{BooleanArray, StringArray, UInt64Array};
use arrow::record_batch::RecordBatch;

use super::{
    validate_julia_plugin_capability_manifest_response_batches,
    julia_plugin_capability_manifest_response_schema,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
};

#[test]
fn capability_manifest_response_validation_rejects_unsupported_transport() {
    let batch = RecordBatch::try_new(
        julia_plugin_capability_manifest_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![Some("xiuxian-julia-core")])),
            Arc::new(StringArray::from(vec![Some("rerank")])),
            Arc::new(StringArray::from(vec![None::<&str>])),
            Arc::new(StringArray::from(vec![Some("http")])),
            Arc::new(StringArray::from(vec![Some("http://127.0.0.1:8815")])),
            Arc::new(StringArray::from(vec![Some("/rerank")])),
            Arc::new(StringArray::from(vec![Some("/healthz")])),
            Arc::new(StringArray::from(vec![Some(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
            )])),
            Arc::new(UInt64Array::from(vec![Some(15)])),
            Arc::new(BooleanArray::from(vec![true])),
        ],
    )
    .unwrap_or_else(|error| panic!("invalid transport batch should build: {error}"));

    let Err(error) = validate_julia_plugin_capability_manifest_response_batches(
        std::slice::from_ref(&batch),
    ) else {
        panic!("unsupported transport should fail");
    };
    assert!(
        error
            .to_string()
            .contains("unsupported `transport_kind` `http`"),
        "unexpected error: {error}"
    );
}
