use std::sync::Arc;

use arrow_array::{RecordBatch, StringArray};
use arrow_flight::FlightDescriptor;
use arrow_flight::flight_service_server::FlightService;
use arrow_schema::{DataType, Field, Schema};
use tonic::metadata::{MetadataMap, MetadataValue};
use xiuxian_security::{
    PublicProtocolSurface, SignedPrincipalSigner, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
};

use super::{WendaoFlightInternalSecurity, WendaoFlightService};

#[test]
fn wendao_flight_internal_security_rejects_missing_gateway_principal() {
    let security = security("internal-secret");

    let error = security
        .verify_metadata(&MetadataMap::new())
        .expect_err("missing Gateway principal should be rejected");

    assert_eq!(error.code(), tonic::Code::Unauthenticated);
    assert!(
        error
            .message()
            .contains("missing internal service identity"),
        "{error:?}"
    );
}

#[test]
fn wendao_flight_internal_security_rejects_raw_public_authorization() {
    let security = security("internal-secret");
    let mut metadata = MetadataMap::new();
    metadata.insert(
        "authorization",
        MetadataValue::from_static("Bearer public-token"),
    );

    let error = security
        .verify_metadata(&metadata)
        .expect_err("raw public Authorization metadata should be rejected");

    assert_eq!(error.code(), tonic::Code::Unauthenticated);
    assert!(error.message().contains("Authorization"), "{error:?}");
}

#[test]
fn wendao_flight_internal_security_accepts_gateway_signed_principal() {
    let security = security("internal-secret");

    security
        .verify_metadata(&gateway_metadata("internal-secret"))
        .unwrap_or_else(|error| panic!("Gateway signed principal should verify: {error}"));
}

#[test]
fn wendao_flight_internal_security_rejects_bad_signature() {
    let security = security("internal-secret");

    let error = security
        .verify_metadata(&gateway_metadata("wrong-secret"))
        .expect_err("wrong signature should be rejected");

    assert_eq!(error.code(), tonic::Code::Unauthenticated);
    assert!(error.message().contains("invalid signed principal"));
}

#[tokio::test]
async fn wendao_flight_service_rejects_get_flight_info_without_gateway_principal() {
    let service = WendaoFlightService::new("1", sample_batch(), 3)
        .unwrap_or_else(|error| panic!("Flight service should build: {error}"))
        .with_internal_security(security("internal-secret"));

    let error = service
        .get_flight_info(tonic::Request::new(FlightDescriptor::new_cmd(
            b"/repo/search".to_vec(),
        )))
        .await
        .expect_err("secured Flight service should reject missing Gateway principal");

    assert_eq!(error.code(), tonic::Code::Unauthenticated);
    assert!(
        error
            .message()
            .contains("WENDAO_FLIGHT_INTERNAL_PRINCIPAL_REQUIRED"),
        "{error:?}"
    );
}

fn security(secret: &str) -> WendaoFlightInternalSecurity {
    WendaoFlightInternalSecurity::gateway(
        Arc::<str>::from(secret),
        Arc::<str>::from("WENDAO_FLIGHT_INTERNAL_PRINCIPAL_REQUIRED"),
    )
}

fn gateway_metadata(signing_secret: &str) -> MetadataMap {
    let surface = PublicProtocolSurface::ArrowFlight;
    let signed_principal = SignedPrincipalSigner::new(
        Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        Arc::<str>::from(signing_secret),
    )
    .sign_user_token(surface, "public-token");

    let mut metadata = MetadataMap::new();
    metadata.insert(
        WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
        MetadataValue::from_static(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
    );
    metadata.insert(
        WENDAO_PUBLIC_PROTOCOL_HEADER,
        MetadataValue::from_static(surface.protocol()),
    );
    metadata.insert(
        WENDAO_AUTH_SCOPE_HEADER,
        MetadataValue::from_static(surface.scope()),
    );
    metadata.insert(
        WENDAO_SIGNED_PRINCIPAL_HEADER,
        MetadataValue::try_from(signed_principal.as_str())
            .unwrap_or_else(|error| panic!("signed principal metadata should build: {error}")),
    );
    metadata
}

fn sample_batch() -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "doc_id",
        DataType::Utf8,
        false,
    )]));
    RecordBatch::try_new(schema, vec![Arc::new(StringArray::from(vec!["doc"]))])
        .unwrap_or_else(|error| panic!("sample batch should build: {error}"))
}
