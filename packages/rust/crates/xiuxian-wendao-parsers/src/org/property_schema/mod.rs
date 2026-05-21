//! Schema gate for Wendao Org reasoning property drawers.

mod diagnostic;
mod records;
mod rules;

pub use diagnostic::OrgReasoningPropertyDiagnostic;
pub use records::{
    ORG_REASONING_PROPERTY_SCHEMA_ID, OrgReasoningPropertyRecord,
    compile_org_reasoning_property_records,
};
pub use rules::{
    ORG_PROP_BLANK_VALUE, ORG_PROP_INVALID_CONFIDENCE, ORG_PROP_INVALID_ENUM,
    ORG_PROP_INVALID_SHA256, ORG_PROP_INVALID_UUID, ORG_PROP_MISSING_REQUIRED,
    ORG_PROP_UNKNOWN_PROPERTY, validate_org_reasoning_properties,
    validate_org_reasoning_property_records,
};
