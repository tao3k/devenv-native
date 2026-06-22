use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, NullArray, StringArray, StringViewArray, UInt64Array, UnionArray,
};
use arrow::buffer::ScalarBuffer;
use arrow::datatypes::{DataType, Field, UnionFields, UnionMode};

use super::{
    sample_response_batch,
    legacy_response_batch,
    legacy_response_batch_with_replaced_column,
    legacy_response_batch_without_health_route,
    legacy_response_batch_without_timeout_secs,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    decode_julia_plugin_capability_manifest_rows,
    validate_julia_plugin_capability_manifest_response_batches,
};

#[test]
fn capability_manifest_decode_rows_materializes_bindings_and_variants() {
    let rows = decode_julia_plugin_capability_manifest_rows(&[sample_response_batch()])
        .unwrap_or_else(|error| panic!("response rows should decode: {error}"));

    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[1].capability_variant.as_deref(),
        Some("structural_rerank")
    );

    let binding = rows[0]
        .to_binding()
        .unwrap_or_else(|error| panic!("enabled row should convert into binding: {error}"))
        .unwrap_or_else(|| panic!("enabled row should produce a binding"));
    assert_eq!(binding.selector, rows[0].selector());
    assert_eq!(binding.endpoint.route.as_deref(), Some("/rerank"));
    assert_eq!(binding.contract_version.0, "v1".to_string());

    let disabled_binding = rows[1]
        .to_binding()
        .unwrap_or_else(|error| panic!("disabled row should still validate: {error}"));
    assert!(disabled_binding.is_none());
}

#[test]
fn capability_manifest_decode_rows_normalizes_legacy_missing_variant_column() {
    let batch = legacy_response_batch(None, None);

    validate_julia_plugin_capability_manifest_response_batches(std::slice::from_ref(&batch))
        .unwrap_or_else(|error| panic!("legacy response should validate: {error}"));

    let rows = decode_julia_plugin_capability_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("legacy response should decode: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].capability_variant, None);
}

#[test]
fn capability_manifest_decode_rows_normalizes_legacy_missing_health_route_column() {
    let batch = legacy_response_batch_without_health_route();

    validate_julia_plugin_capability_manifest_response_batches(std::slice::from_ref(&batch))
        .unwrap_or_else(|error| panic!("legacy response should validate: {error}"));

    let rows = decode_julia_plugin_capability_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("legacy response should decode: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].health_route, None);
}

#[test]
fn capability_manifest_decode_rows_normalizes_legacy_missing_timeout_secs_column() {
    let batch = legacy_response_batch_without_timeout_secs();

    validate_julia_plugin_capability_manifest_response_batches(std::slice::from_ref(&batch))
        .unwrap_or_else(|error| panic!("legacy response should validate: {error}"));

    let rows = decode_julia_plugin_capability_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("legacy response should decode: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].timeout_secs, None);
}

#[test]
fn capability_manifest_decode_rows_normalizes_legacy_null_variant_column() {
    let batch = legacy_response_batch(
        Some(Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
            DataType::Null,
            true,
        )),
        Some(Arc::new(NullArray::new(1))),
    );

    validate_julia_plugin_capability_manifest_response_batches(std::slice::from_ref(&batch))
        .unwrap_or_else(|error| panic!("legacy null response should validate: {error}"));

    let rows = decode_julia_plugin_capability_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("legacy null response should decode: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].capability_variant, None);
}

#[test]
fn capability_manifest_decode_rows_normalizes_legacy_view_variant_column() {
    let batch = legacy_response_batch(
        Some(Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
            DataType::Utf8View,
            true,
        )),
        Some(Arc::new(StringViewArray::from(vec![Some("structural_rerank")]))),
    );

    validate_julia_plugin_capability_manifest_response_batches(std::slice::from_ref(&batch))
        .unwrap_or_else(|error| panic!("legacy view response should validate: {error}"));

    let rows = decode_julia_plugin_capability_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("legacy view response should decode: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].capability_variant.as_deref(),
        Some("structural_rerank")
    );
}

#[test]
fn capability_manifest_decode_rows_normalizes_julia_nothing_string_union_variant_column() {
    let mut nothing_field = Field::new("", DataType::Null, true);
    nothing_field.set_metadata(HashMap::from([
        (
            "ARROW:extension:name".to_string(),
            "JuliaLang.Nothing".to_string(),
        ),
        ("ARROW:extension:metadata".to_string(), String::new()),
    ]));
    let union_fields = UnionFields::from_iter([
        (0, Arc::new(Field::new("", DataType::Null, true))),
        (1, Arc::new(nothing_field)),
        (2, Arc::new(Field::new("", DataType::Utf8, false))),
    ]);
    let column = UnionArray::try_new(
        union_fields.clone(),
        ScalarBuffer::from(vec![1_i8, 2]),
        Some(ScalarBuffer::from(vec![0_i32, 0])),
        vec![
            Arc::new(NullArray::new(1)) as ArrayRef,
            Arc::new(NullArray::new(1)) as ArrayRef,
            Arc::new(StringArray::from(vec![Some("structural_rerank")])) as ArrayRef,
        ],
    )
    .unwrap_or_else(|error| panic!("Julia optional string union should build: {error}"));
    let batch = legacy_response_batch(
        Some(Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
            DataType::Union(union_fields, UnionMode::Dense),
            false,
        )),
        Some(Arc::new(column)),
    );

    validate_julia_plugin_capability_manifest_response_batches(std::slice::from_ref(&batch))
        .unwrap_or_else(|error| panic!("Julia union response should validate: {error}"));

    let rows = decode_julia_plugin_capability_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("Julia union response should decode: {error}"));

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].capability_variant, None);
    assert_eq!(
        rows[1].capability_variant.as_deref(),
        Some("structural_rerank")
    );
}

#[test]
fn capability_manifest_decode_rows_normalizes_julia_nothing_u64_union_timeout_column() {
    let mut nothing_field = Field::new("", DataType::Null, true);
    nothing_field.set_metadata(HashMap::from([
        (
            "ARROW:extension:name".to_string(),
            "JuliaLang.Nothing".to_string(),
        ),
        ("ARROW:extension:metadata".to_string(), String::new()),
    ]));
    let union_fields = UnionFields::from_iter([
        (0, Arc::new(Field::new("", DataType::Null, true))),
        (1, Arc::new(nothing_field)),
        (2, Arc::new(Field::new("", DataType::UInt64, false))),
    ]);
    let column = UnionArray::try_new(
        union_fields.clone(),
        ScalarBuffer::from(vec![1_i8, 2]),
        Some(ScalarBuffer::from(vec![0_i32, 0])),
        vec![
            Arc::new(NullArray::new(1)) as ArrayRef,
            Arc::new(NullArray::new(1)) as ArrayRef,
            Arc::new(UInt64Array::from(vec![Some(42_u64)])) as ArrayRef,
        ],
    )
    .unwrap_or_else(|error| panic!("Julia optional UInt64 union should build: {error}"));
    let batch = legacy_response_batch_with_replaced_column(
        JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
            DataType::Union(union_fields, UnionMode::Dense),
            false,
        ),
        Arc::new(column),
    );

    validate_julia_plugin_capability_manifest_response_batches(std::slice::from_ref(&batch))
        .unwrap_or_else(|error| panic!("Julia UInt64 union response should validate: {error}"));

    let rows = decode_julia_plugin_capability_manifest_rows(&[batch])
        .unwrap_or_else(|error| panic!("Julia UInt64 union response should decode: {error}"));

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].timeout_secs, None);
    assert_eq!(rows[1].timeout_secs, Some(42_u64.into()));
}
