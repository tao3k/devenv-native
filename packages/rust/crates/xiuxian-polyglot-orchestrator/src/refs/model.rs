//! Typed references to external owner contracts.

use serde::{Deserialize, Serialize};

use crate::lanes::PolyglotLane;

/// Package boundary that owns an external contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractOwner {
    /// Runtime-owned deployment and transport configuration.
    #[serde(rename = "wendao_runtime")]
    Runtime,
    /// Attachment-owned OCR and cache contracts.
    #[serde(rename = "wendao_attachments")]
    Attachments,
    /// Julia-owned profile, schema, and readiness contracts.
    #[serde(rename = "wendao_julia")]
    Julia,
    /// Analyzer-owned Python Docling worker routes.
    #[serde(rename = "wendao_analyzer")]
    Analyzer,
}

/// Typed reference to an existing route, profile, or schema owner.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteProfileRef {
    /// Lane that uses the referenced contract.
    pub lane: PolyglotLane,
    /// Package that owns the referenced contract.
    pub owner: ContractOwner,
    /// Route, capability, or profile route identifier.
    pub route: String,
    /// Optional profile identifier.
    pub profile: Option<String>,
    /// Optional schema version identifier.
    pub schema_version: Option<String>,
}

impl RouteProfileRef {
    /// Creates a reference to the existing analyzer document-extract route.
    #[must_use]
    pub fn document_extract(route: impl Into<String>) -> Self {
        Self {
            lane: PolyglotLane::PythonDocling,
            owner: ContractOwner::Analyzer,
            route: route.into(),
            profile: None,
            schema_version: None,
        }
    }

    /// Creates a reference to the existing attachment OCR shard route contract.
    #[must_use]
    pub fn ocr_shards(route: impl Into<String>, schema_version: impl Into<String>) -> Self {
        Self {
            lane: PolyglotLane::PythonDocling,
            owner: ContractOwner::Attachments,
            route: route.into(),
            profile: None,
            schema_version: Some(schema_version.into()),
        }
    }

    /// Creates a reference to a Julia profile route contract.
    #[must_use]
    pub fn julia_profile(
        route: impl Into<String>,
        profile: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Self {
        Self {
            lane: PolyglotLane::JuliaCompute,
            owner: ContractOwner::Julia,
            route: route.into(),
            profile: Some(profile.into()),
            schema_version: Some(schema_version.into()),
        }
    }
}
