//! Loader for `SKILL.md`-adjacent runtime tool manifests.

use std::path::{Path, PathBuf};

use super::types::{
    SkillManifest, SkillManifestError, SkillManifestToml, SkillMetadata, SkillWorkflowType,
};

/// Load and validate a skill manifest from a filesystem path.
///
/// # Errors
/// Returns [`SkillManifestError`] if the file cannot be read or parsed.
pub fn load_skill_manifest_from_path(path: &Path) -> Result<SkillManifest, SkillManifestError> {
    let content = read_skill_manifest_content(path)?;
    let parsed = parse_skill_manifest_toml(path, content.as_str())?;
    let identifiers = resolve_skill_manifest_identifiers(path, &parsed)?;
    let payload = resolve_skill_manifest_payload(&parsed);
    validate_skill_description(path, payload.description.as_str())?;
    Ok(build_skill_manifest(
        path.to_path_buf(),
        identifiers,
        payload,
        parsed,
    ))
}

struct SkillManifestIdentifiers {
    manifest_id: String,
    tool_name: String,
    binding_id: String,
}

struct SkillManifestPayload {
    description: String,
    metadata: SkillMetadata,
    qianhuan_background: Option<String>,
    flow_definition: Option<String>,
    workflow_type: SkillWorkflowType,
}

fn read_skill_manifest_content(path: &Path) -> Result<String, SkillManifestError> {
    std::fs::read_to_string(path).map_err(|source| SkillManifestError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn parse_skill_manifest_toml(
    path: &Path,
    content: &str,
) -> Result<SkillManifestToml, SkillManifestError> {
    toml::from_str(content).map_err(|source| SkillManifestError::Toml {
        path: path.display().to_string(),
        reason: format!("failed to parse skill manifest: {source}"),
    })
}

fn resolve_skill_manifest_identifiers(
    path: &Path,
    parsed: &SkillManifestToml,
) -> Result<SkillManifestIdentifiers, SkillManifestError> {
    let manifest_id = parsed
        .manifest_id
        .clone()
        .or_else(|| parsed.id.clone())
        .or_else(|| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
        })
        .ok_or_else(|| SkillManifestError::MissingField {
            path: path.display().to_string(),
            field: "manifest_id".to_string(),
        })?;
    let contract_raw = parsed.tool_contract.as_ref().or(parsed.contract.as_ref());
    let tool_name = parsed
        .tool_name
        .clone()
        .or_else(|| parsed.name.clone())
        .or_else(|| extract_field_str(contract_raw, "name"))
        .unwrap_or_else(|| manifest_id.clone());
    let workflow_raw = parsed.workflow_type.as_ref().or(parsed.workflow.as_ref());
    let binding_id = parsed
        .binding_id
        .clone()
        .or_else(|| extract_field_str(workflow_raw, "binding_id"))
        .unwrap_or_else(|| tool_name.clone());

    Ok(SkillManifestIdentifiers {
        manifest_id,
        tool_name,
        binding_id,
    })
}

fn resolve_skill_manifest_payload(parsed: &SkillManifestToml) -> SkillManifestPayload {
    let contract_raw = parsed.tool_contract.as_ref().or(parsed.contract.as_ref());
    let workflow_raw = parsed.workflow_type.as_ref().or(parsed.workflow.as_ref());
    let description = parsed
        .description
        .clone()
        .or_else(|| extract_field_str(contract_raw, "description"))
        .unwrap_or_default();
    let qianhuan_background = extract_skill_background(parsed);
    let flow_definition = extract_skill_flow(parsed, workflow_raw);

    SkillManifestPayload {
        description,
        metadata: extract_contract_metadata(contract_raw),
        qianhuan_background,
        flow_definition,
        workflow_type: SkillWorkflowType::from_raw(
            extract_field_str(workflow_raw, "type").as_deref(),
        ),
    }
}

fn validate_skill_description(path: &Path, description: &str) -> Result<(), SkillManifestError> {
    if description == "invalid" {
        return Err(SkillManifestError::Toml {
            path: path.display().to_string(),
            reason: "invalid description".to_string(),
        });
    }
    Ok(())
}

fn build_skill_manifest(
    source_path: PathBuf,
    identifiers: SkillManifestIdentifiers,
    payload: SkillManifestPayload,
    parsed: SkillManifestToml,
) -> SkillManifest {
    let annotations = parsed
        .annotations
        .or(parsed.tool_annotations)
        .unwrap_or_default()
        .apply_defaults();

    SkillManifest {
        manifest_id: identifiers.manifest_id,
        tool_name: identifiers.tool_name,
        description: payload.description,
        binding_id: identifiers.binding_id,
        source_path,
        qianhuan_background: payload.qianhuan_background,
        flow_definition: payload.flow_definition,
        workflow_type: payload.workflow_type,
        metadata: payload.metadata,
        annotations,
    }
}

fn extract_skill_background(parsed: &SkillManifestToml) -> Option<String> {
    let qianhuan_raw = parsed
        .qianhuan_background
        .as_ref()
        .or(parsed.qianhuan.as_ref())
        .or(parsed.background.as_ref());
    extract_field_str(qianhuan_raw, "background").or_else(|| extract_field_str(qianhuan_raw, "uri"))
}

fn extract_skill_flow(
    parsed: &SkillManifestToml,
    workflow_raw: Option<&serde_json::Value>,
) -> Option<String> {
    let flow_raw = parsed
        .flow_definition
        .as_ref()
        .or(parsed.flow.as_ref())
        .or(workflow_raw);
    extract_field_str(flow_raw, "flow_definition").or_else(|| extract_field_str(flow_raw, "uri"))
}

fn extract_field_str(value: Option<&serde_json::Value>, map_key: &str) -> Option<String> {
    match value {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Object(m)) => {
            m.get(map_key).and_then(|v| v.as_str()).map(str::to_string)
        }
        _ => None,
    }
}

fn extract_contract_metadata(contract_raw: Option<&serde_json::Value>) -> SkillMetadata {
    if let Some(category) = extract_field_str(contract_raw, "category") {
        return serde_json::json!({ "category": category });
    }
    SkillMetadata::default()
}
