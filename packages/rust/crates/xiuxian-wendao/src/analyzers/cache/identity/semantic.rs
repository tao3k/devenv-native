use std::path::Path;

#[cfg(feature = "julia")]
use xiuxian_julia_core::modelica_parser_summary_file_semantic_fingerprint_for_repository;

use crate::analyzers::RegisteredRepository;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SemanticFingerprintOwner {
    JuliaParserSummary,
    ModelicaParserSummary,
    GenericCode(String),
}

impl SemanticFingerprintOwner {
    pub(super) fn mode_label(&self) -> String {
        match self {
            Self::JuliaParserSummary => "semantic:julia_parser_summary".to_string(),
            Self::ModelicaParserSummary => "semantic:modelica_parser_summary".to_string(),
            Self::GenericCode(language_id) => {
                format!("semantic:language_provider:{}", language_id.as_str())
            }
        }
    }
}

fn plugin_id_supports_semantic_owner_dispatch(plugin_id: &str) -> bool {
    matches!(plugin_id, "julia-code-parser" | "modelica")
        || source_language_id_from_identifier(plugin_id).is_some()
}

pub(crate) fn plugin_ids_allow_semantic_owner_dispatch(plugin_ids: &[String]) -> bool {
    plugin_ids.is_empty()
        || plugin_ids
            .iter()
            .all(|plugin_id| plugin_id_supports_semantic_owner_dispatch(plugin_id))
}

pub(crate) fn semantic_fingerprint_for_file(
    repository: &RegisteredRepository,
    relative_path: &str,
    source_text: &str,
    plugin_ids: &[String],
) -> Option<String> {
    let owner = semantic_fingerprint_owner(relative_path, plugin_ids)?;
    compute_semantic_fingerprint(&owner, repository, relative_path, source_text)
}

pub(crate) fn plugin_ids_support_semantic_owner_reuse(plugin_ids: &[String]) -> bool {
    !plugin_ids.is_empty() && plugin_ids_allow_semantic_owner_dispatch(plugin_ids)
}

pub(super) fn semantic_fingerprint_owner(
    relative_path: &str,
    plugin_ids: &[String],
) -> Option<SemanticFingerprintOwner> {
    if !plugin_ids_allow_semantic_owner_dispatch(plugin_ids) {
        return None;
    }

    if plugin_ids
        .iter()
        .any(|plugin_id| plugin_id == "julia-code-parser")
        && relative_path.starts_with("src/")
        && has_extension(relative_path, "jl")
    {
        return Some(SemanticFingerprintOwner::JuliaParserSummary);
    }
    if plugin_ids.iter().any(|plugin_id| plugin_id == "modelica")
        && has_extension(relative_path, "mo")
    {
        return Some(SemanticFingerprintOwner::ModelicaParserSummary);
    }

    let language_id = source_language_id_from_path(Path::new(relative_path))?;
    Some(SemanticFingerprintOwner::GenericCode(language_id))
}

pub(super) fn compute_semantic_fingerprint(
    owner: &SemanticFingerprintOwner,
    repository: &RegisteredRepository,
    relative_path: &str,
    source_text: &str,
) -> Option<String> {
    match owner {
        SemanticFingerprintOwner::JuliaParserSummary => {
            julia_source_semantic_fingerprint(source_text)
        }
        SemanticFingerprintOwner::ModelicaParserSummary => {
            modelica_parser_summary_semantic_fingerprint(repository, relative_path, source_text)
        }
        SemanticFingerprintOwner::GenericCode(language_id) => {
            generic_source_semantic_fingerprint(source_text, language_id.as_str())
        }
    }
}

fn source_language_id_from_identifier(identifier: &str) -> Option<String> {
    match identifier.trim().to_ascii_lowercase().as_str() {
        "rust" | "rs" | "rust-lang-parser" => Some("rust".to_owned()),
        "python" | "py" | "python-lang-parser" => Some("python".to_owned()),
        "typescript" | "ts" | "tsx" | "typescript-lang-parser" => Some("typescript".to_owned()),
        "javascript" | "js" | "jsx" | "javascript-lang-parser" => Some("javascript".to_owned()),
        "julia" | "jl" | "julia-lang-parser" => Some("julia".to_owned()),
        "modelica" | "mo" => Some("modelica".to_owned()),
        _ => None,
    }
}

fn source_language_id_from_path(path: &Path) -> Option<String> {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some(ext) if ext.eq_ignore_ascii_case("rs") => Some("rust".to_owned()),
        Some(ext) if ext.eq_ignore_ascii_case("py") => Some("python".to_owned()),
        Some(ext) if ext.eq_ignore_ascii_case("ts") || ext.eq_ignore_ascii_case("tsx") => {
            Some("typescript".to_owned())
        }
        Some(ext) if ext.eq_ignore_ascii_case("js") || ext.eq_ignore_ascii_case("jsx") => {
            Some("javascript".to_owned())
        }
        Some(ext) if ext.eq_ignore_ascii_case("jl") => Some("julia".to_owned()),
        Some(ext) if ext.eq_ignore_ascii_case("mo") => Some("modelica".to_owned()),
        _ => None,
    }
}

fn generic_source_semantic_fingerprint(source_text: &str, language_id: &str) -> Option<String> {
    let normalized = source_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if normalized.is_empty() {
        return None;
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xiuxian_wendao.language_provider_semantic_fingerprint.v1\0");
    hasher.update(language_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(normalized.as_bytes());
    Some(hasher.finalize().to_hex().to_string())
}

fn modelica_parser_summary_semantic_fingerprint(
    repository: &RegisteredRepository,
    relative_path: &str,
    source_text: &str,
) -> Option<String> {
    #[cfg(feature = "julia")]
    {
        modelica_parser_summary_file_semantic_fingerprint_for_repository(
            repository,
            relative_path.into(),
            source_text,
        )
        .ok()
    }

    #[cfg(not(feature = "julia"))]
    {
        let _ = (repository, relative_path, source_text);
        None
    }
}

fn julia_source_semantic_fingerprint(source_text: &str) -> Option<String> {
    let normalized = source_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    if normalized.is_empty() {
        return None;
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"xiuxian_wendao.julia_source_semantic_fingerprint.v1\0");
    hasher.update(normalized.as_bytes());
    Some(hasher.finalize().to_hex().to_string())
}

fn has_extension(relative_path: &str, extension: &str) -> bool {
    Path::new(relative_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|current| current.eq_ignore_ascii_case(extension))
}
