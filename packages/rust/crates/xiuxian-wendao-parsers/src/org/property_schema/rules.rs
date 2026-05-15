//! Validation rules for schema-governed Org reasoning properties.

use uuid::Uuid;

use super::super::ontology::OrgOntologyAuthoringDocument;
use super::diagnostic::OrgReasoningPropertyDiagnostic;
use super::records::{OrgReasoningPropertyRecord, compile_org_reasoning_property_records};

/// Missing required property.
pub const ORG_PROP_MISSING_REQUIRED: &str = "ORG-PROP-001";
/// Invalid UUID property value.
pub const ORG_PROP_INVALID_UUID: &str = "ORG-PROP-002";
/// Unknown property key.
pub const ORG_PROP_UNKNOWN_PROPERTY: &str = "ORG-PROP-003";
/// Invalid enum property value.
pub const ORG_PROP_INVALID_ENUM: &str = "ORG-PROP-004";
/// Invalid confidence property value.
pub const ORG_PROP_INVALID_CONFIDENCE: &str = "ORG-PROP-005";
/// Invalid SHA-256 property value.
pub const ORG_PROP_INVALID_SHA256: &str = "ORG-PROP-006";
/// Blank property value.
pub const ORG_PROP_BLANK_VALUE: &str = "ORG-PROP-007";

const ALLOWED_PROPERTIES: &[&str] = &[
    "AUTHORING_KIND",
    "CONFIDENCE",
    "CREATED_AT",
    "DOMAIN",
    "EVIDENCE_ID",
    "ID",
    "KIND",
    "LIFECYCLE_STATE",
    "MAPPING_ID",
    "MODEL_ID",
    "NOTES",
    "ONTOLOGY_KIND",
    "PRIMARY_LANGUAGE",
    "PROMOTION_STATE",
    "REVIEWER",
    "SOURCE_HANDLE",
    "SOURCE_SHA256",
    "STATE",
    "STATUS",
    "UPDATED_AT",
    "VALIDATION_TARGET",
    "WENDAO_KEY",
    "WENDAO_KIND",
];

const WENDAO_KINDS: &[&str] = &[
    "ontology_mapping",
    "evidence_summary",
    "validation_feedback",
];

const PROMOTION_STATES: &[&str] = &[
    "draft",
    "candidate",
    "validated",
    "promoted",
    "rejected",
    "blocked",
];

/// Validate schema-governed Org reasoning properties in a compiled authoring
/// document.
#[must_use]
pub fn validate_org_reasoning_properties(
    document: &OrgOntologyAuthoringDocument,
) -> Vec<OrgReasoningPropertyDiagnostic> {
    let records = compile_org_reasoning_property_records(document);
    validate_org_reasoning_property_records(&records)
}

/// Validate compiled Org reasoning property records.
#[must_use]
pub fn validate_org_reasoning_property_records(
    records: &[OrgReasoningPropertyRecord],
) -> Vec<OrgReasoningPropertyDiagnostic> {
    records.iter().flat_map(validate_record).collect()
}

fn validate_record(record: &OrgReasoningPropertyRecord) -> Vec<OrgReasoningPropertyDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_allowed_properties(record, &mut diagnostics);
    validate_no_blank_values(record, &mut diagnostics);
    validate_required(record, "ID", &mut diagnostics);
    validate_required(record, "WENDAO_KIND", &mut diagnostics);

    if let Some(id) = record.properties.get("ID") {
        validate_uuid(record, "ID", id, &mut diagnostics);
    }

    let Some(kind) = record.properties.get("WENDAO_KIND") else {
        return diagnostics;
    };

    validate_enum(record, "WENDAO_KIND", kind, WENDAO_KINDS, &mut diagnostics);
    match kind.as_str() {
        "ontology_mapping" => {
            validate_required(record, "PROMOTION_STATE", &mut diagnostics);
        }
        "evidence_summary" => {
            validate_required(record, "SOURCE_HANDLE", &mut diagnostics);
        }
        "validation_feedback" => {
            validate_required(record, "VALIDATION_TARGET", &mut diagnostics);
        }
        _ => {}
    }

    if let Some(state) = record.properties.get("PROMOTION_STATE") {
        validate_enum(
            record,
            "PROMOTION_STATE",
            state,
            PROMOTION_STATES,
            &mut diagnostics,
        );
    }
    if let Some(confidence) = record.properties.get("CONFIDENCE") {
        validate_confidence(record, confidence, &mut diagnostics);
    }
    if let Some(hash) = record.properties.get("SOURCE_SHA256") {
        validate_sha256(record, hash, &mut diagnostics);
    }

    diagnostics
}

fn validate_allowed_properties(
    record: &OrgReasoningPropertyRecord,
    diagnostics: &mut Vec<OrgReasoningPropertyDiagnostic>,
) {
    for property in record.properties.keys() {
        if !ALLOWED_PROPERTIES.contains(&property.as_str()) {
            diagnostics.push(diagnostic(
                ORG_PROP_UNKNOWN_PROPERTY,
                format!("Org property `{property}` is not allowed by the Wendao reasoning property schema"),
                record,
                Some(property),
            ));
        }
    }
}

fn validate_no_blank_values(
    record: &OrgReasoningPropertyRecord,
    diagnostics: &mut Vec<OrgReasoningPropertyDiagnostic>,
) {
    for (property, value) in &record.properties {
        if value.trim().is_empty() {
            diagnostics.push(diagnostic(
                ORG_PROP_BLANK_VALUE,
                format!("Org property `{property}` must not be blank"),
                record,
                Some(property),
            ));
        }
    }
}

fn validate_required(
    record: &OrgReasoningPropertyRecord,
    property: &str,
    diagnostics: &mut Vec<OrgReasoningPropertyDiagnostic>,
) {
    if record
        .properties
        .get(property)
        .is_none_or(|value| value.trim().is_empty())
    {
        diagnostics.push(diagnostic(
            ORG_PROP_MISSING_REQUIRED,
            format!(
                "Org property `{property}` is required by the Wendao reasoning property schema"
            ),
            record,
            Some(property),
        ));
    }
}

fn validate_uuid(
    record: &OrgReasoningPropertyRecord,
    property: &str,
    value: &str,
    diagnostics: &mut Vec<OrgReasoningPropertyDiagnostic>,
) {
    if Uuid::parse_str(value).is_err() {
        diagnostics.push(diagnostic(
            ORG_PROP_INVALID_UUID,
            format!("Org property `{property}` must be a UUID"),
            record,
            Some(property),
        ));
    }
}

fn validate_enum(
    record: &OrgReasoningPropertyRecord,
    property: &str,
    value: &str,
    allowed: &[&str],
    diagnostics: &mut Vec<OrgReasoningPropertyDiagnostic>,
) {
    if !allowed.contains(&value) {
        diagnostics.push(diagnostic(
            ORG_PROP_INVALID_ENUM,
            format!(
                "Org property `{property}` has invalid value `{value}`; expected one of {}",
                allowed.join(", ")
            ),
            record,
            Some(property),
        ));
    }
}

fn validate_confidence(
    record: &OrgReasoningPropertyRecord,
    value: &str,
    diagnostics: &mut Vec<OrgReasoningPropertyDiagnostic>,
) {
    let valid = value
        .parse::<f64>()
        .is_ok_and(|parsed| parsed.is_finite() && (0.0..=1.0).contains(&parsed));
    if !valid {
        diagnostics.push(diagnostic(
            ORG_PROP_INVALID_CONFIDENCE,
            "Org property `CONFIDENCE` must be a number between 0 and 1".to_string(),
            record,
            Some("CONFIDENCE"),
        ));
    }
}

fn validate_sha256(
    record: &OrgReasoningPropertyRecord,
    value: &str,
    diagnostics: &mut Vec<OrgReasoningPropertyDiagnostic>,
) {
    if !is_sha256(value) {
        diagnostics.push(diagnostic(
            ORG_PROP_INVALID_SHA256,
            "Org property `SOURCE_SHA256` must be a SHA-256 hex digest".to_string(),
            record,
            Some("SOURCE_SHA256"),
        ));
    }
}

fn is_sha256(value: &str) -> bool {
    let digest = value.strip_prefix("sha256:").unwrap_or(value);
    digest.len() == 64
        && digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn diagnostic(
    code: &str,
    message: String,
    record: &OrgReasoningPropertyRecord,
    property: Option<&str>,
) -> OrgReasoningPropertyDiagnostic {
    OrgReasoningPropertyDiagnostic {
        code: code.to_string(),
        message,
        document_id: record.document_id.clone(),
        section_id: record.section_id.clone(),
        heading_path: record.heading_path.clone(),
        source_path: record.source_path.clone(),
        property: property.map(ToString::to_string),
        source_span: record.source_span.clone(),
    }
}
