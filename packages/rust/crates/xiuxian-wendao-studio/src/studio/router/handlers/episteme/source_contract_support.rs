use std::path::{Path, PathBuf};

use xiuxian_wendao::episteme::{
    EpistemeError, EpistemeRegistryEntry, EpistemeRegistryError, EpistemeRuntimeConfig,
    configured_episteme_corpus_root_env, load_episteme_registry_entries,
    load_episteme_runtime_config, validate_episteme_registry_reference_graph,
};

use crate::studio::router::{StudioApiError, load_episteme_registry_from_wendao_toml};

pub(super) trait EpistemeRootRequest {
    fn episteme_root(&self) -> Option<&str>;
    fn episteme_registry_id(&self) -> Option<&str>;
}

pub(super) fn load_runtime_config(
    episteme_root: &Path,
) -> Result<Option<EpistemeRuntimeConfig>, StudioApiError> {
    load_episteme_runtime_config(episteme_root).map_err(map_episteme_source_contract_error)
}

pub(super) fn resolve_episteme_root(
    project_root: &Path,
    config_root: &Path,
    request: &impl EpistemeRootRequest,
) -> Result<PathBuf, StudioApiError> {
    match (
        trimmed_optional(request.episteme_root()),
        trimmed_optional(request.episteme_registry_id()),
    ) {
        (Some(episteme_root), None) => {
            Ok(resolve_project_path(project_root, episteme_root.as_str()))
        }
        (None, Some(registry_id)) => {
            resolve_episteme_registry_root(project_root, config_root, registry_id.as_str())
        }
        (Some(_), Some(_)) => Err(StudioApiError::bad_request(
            "AMBIGUOUS_EPISTEME_SOURCE",
            "`epistemeRoot` and `epistemeRegistryId` are mutually exclusive",
        )),
        (None, None) => Err(StudioApiError::bad_request(
            "MISSING_EPISTEME_SOURCE",
            "`epistemeRoot` or `epistemeRegistryId` is required",
        )),
    }
}

pub(super) fn resolve_corpus_root(
    project_root: &Path,
    episteme_root: &Path,
    raw: Option<&str>,
    runtime_config: Option<&EpistemeRuntimeConfig>,
) -> Result<PathBuf, StudioApiError> {
    if let Some(value) = trimmed_optional(raw) {
        return Ok(resolve_project_path(project_root, value.as_str()));
    }
    if let Some(path) = runtime_config.and_then(|config| config.corpus.clone()) {
        return Ok(path);
    }
    let corpus_root_env = configured_episteme_corpus_root_env(episteme_root)
        .map_err(map_episteme_source_contract_error)?;
    let value = std::env::var(corpus_root_env.as_str()).map_err(|_| {
        StudioApiError::bad_request(
            "MISSING_CORPUS_ROOT",
            format!("`corpusRoot` is required when {corpus_root_env} is not set"),
        )
    })?;
    resolve_required_path(project_root, value.as_str(), "corpusRoot")
}

pub(super) fn resolve_run_root(
    project_root: &Path,
    raw: Option<&str>,
    configured: Option<&Path>,
    default: impl FnOnce() -> PathBuf,
) -> PathBuf {
    if let Some(value) = trimmed_optional(raw) {
        return resolve_project_path(project_root, value.as_str());
    }
    if let Some(path) = configured {
        return path.to_path_buf();
    }
    default()
}

pub(super) fn trimmed_required<'a>(raw: &'a str, field: &str) -> Result<&'a str, StudioApiError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        let code = format!("MISSING_{}", screaming_snake_case(field));
        return Err(StudioApiError::bad_request(
            code.as_str(),
            format!("`{field}` is required"),
        ));
    }
    Ok(trimmed)
}

pub(super) fn trimmed_optional(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn map_episteme_source_contract_error(error: EpistemeError) -> StudioApiError {
    match error {
        EpistemeError::InvalidRunId(_)
        | EpistemeError::InvalidEpistemeManifest(_)
        | EpistemeError::InvalidContract(_)
        | EpistemeError::EmptySelection
        | EpistemeError::Parse { .. }
        | EpistemeError::EpistemeManifestToml { .. } => StudioApiError::bad_request(
            "EPISTEME_SOURCE_CONTRACT_RUN_PLAN_REJECTED",
            error.to_string(),
        ),
        EpistemeError::Io { path, source } => {
            if source.kind() == std::io::ErrorKind::NotFound {
                StudioApiError::bad_request(
                    "EPISTEME_SOURCE_CONTRACT_PATH_NOT_FOUND",
                    format!(
                        "episteme source-contract path was not found: `{}`",
                        path.display()
                    ),
                )
            } else {
                StudioApiError::internal(
                    "EPISTEME_SOURCE_CONTRACT_RUN_PLAN_IO_FAILED",
                    "Failed to write episteme source-contract run-plan artifacts",
                    Some(format!("{}: {source}", path.display())),
                )
            }
        }
        EpistemeError::Json { path, source } => StudioApiError::internal(
            "EPISTEME_SOURCE_CONTRACT_RUN_PLAN_JSON_FAILED",
            "Failed to serialize episteme source-contract run-plan receipt",
            Some(format!("{}: {source}", path.display())),
        ),
        EpistemeError::ReadModel(detail) => StudioApiError::internal(
            "EPISTEME_SOURCE_CONTRACT_READ_MODEL_FAILED",
            "Failed to materialize episteme source-contract read-model seed",
            Some(detail),
        ),
    }
}

fn resolve_episteme_registry_root(
    project_root: &Path,
    config_root: &Path,
    registry_id: &str,
) -> Result<PathBuf, StudioApiError> {
    let entries = load_episteme_registry_from_wendao_toml(config_root).map_err(|error| {
        StudioApiError::bad_request(
            "EPISTEME_REGISTRY_CONFIG_INVALID",
            format!("failed to load episteme registry config: {error}"),
        )
    })?;
    let Some(entry) = find_episteme_registry_entry(entries.as_slice(), registry_id) else {
        return Err(StudioApiError::bad_request(
            "EPISTEME_REGISTRY_NOT_FOUND",
            format!("episteme registry `{registry_id}` is not configured"),
        ));
    };
    if !entry.enabled {
        return Err(StudioApiError::bad_request(
            "EPISTEME_REGISTRY_DISABLED",
            format!("episteme registry `{registry_id}` is disabled"),
        ));
    }
    let receipt = load_episteme_registry_entries(entries.as_slice(), project_root)
        .map_err(|error| map_episteme_registry_error(&error))?;
    validate_episteme_registry_reference_graph(&receipt)
        .map_err(|error| map_episteme_registry_error(&error))?;
    receipt
        .entries
        .into_iter()
        .find(|entry| entry.id == registry_id)
        .map(|entry| entry.episteme_root)
        .ok_or_else(|| {
            StudioApiError::bad_request(
                "EPISTEME_REGISTRY_NOT_LOADED",
                format!("episteme registry `{registry_id}` did not load an episteme root"),
            )
        })
}

fn find_episteme_registry_entry<'a>(
    entries: &'a [EpistemeRegistryEntry],
    registry_id: &str,
) -> Option<&'a EpistemeRegistryEntry> {
    entries.iter().find(|entry| entry.id == registry_id)
}

fn resolve_required_path(
    project_root: &Path,
    raw: &str,
    field: &str,
) -> Result<PathBuf, StudioApiError> {
    let trimmed = trimmed_required(raw, field)?;
    Ok(resolve_project_path(project_root, trimmed))
}

fn resolve_project_path(project_root: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}

fn screaming_snake_case(field: &str) -> String {
    let mut output = String::new();
    for (index, character) in field.chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            output.push('_');
        }
        output.push(character.to_ascii_uppercase());
    }
    output
}

fn map_episteme_registry_error(error: &EpistemeRegistryError) -> StudioApiError {
    StudioApiError::bad_request("EPISTEME_REGISTRY_LOAD_REJECTED", error.to_string())
}
