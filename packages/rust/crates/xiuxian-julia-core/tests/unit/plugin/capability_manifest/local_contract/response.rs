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

    let Err(error) = validate_julia_plugin_capability_manifest_response_batches(&[batch]) else {
        panic!("unsupported transport should fail");
    };
    assert!(
        error
            .to_string()
            .contains("unsupported `transport_kind` `http`"),
        "unexpected error: {error}"
    );
}

fn legacy_response_batch(
    capability_variant_field: Option<Field>,
    capability_variant_column: Option<Arc<dyn arrow::array::Array>>,
) -> RecordBatch {
    let mut fields = vec![
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
            DataType::UInt64,
            true,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
            DataType::Boolean,
            false,
        ),
    ];
    let mut columns: Vec<Arc<dyn arrow::array::Array>> = vec![
        Arc::new(StringArray::from(vec![Some(JULIA_PLUGIN_ID)])),
        Arc::new(StringArray::from(vec![Some("rerank")])),
        Arc::new(StringArray::from(vec![Some("arrow_flight")])),
        Arc::new(StringArray::from(vec![Some("http://127.0.0.1:8815")])),
        Arc::new(StringArray::from(vec![Some("/rerank")])),
        Arc::new(StringArray::from(vec![Some("/healthz")])),
        Arc::new(StringArray::from(vec![Some("v0-draft")])),
        Arc::new(UInt64Array::from(vec![Some(15)])),
        Arc::new(BooleanArray::from(vec![true])),
    ];

    if let Some(field) = capability_variant_field {
        fields.insert(2, field);
    }
    if let Some(column) = capability_variant_column {
        columns.insert(2, column);
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .unwrap_or_else(|error| panic!("legacy response batch should build: {error}"))
}
