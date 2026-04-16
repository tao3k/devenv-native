use std::path::Path;

use xiuxian_ast::{
    Lang, semantic_fingerprint as generic_ast_semantic_fingerprint, supports_semantic_fingerprint,
};
use xiuxian_wendao_julia::{
    julia_parser_summary_file_semantic_fingerprint_for_repository,
    modelica_parser_summary_file_semantic_fingerprint_for_repository,
};

use crate::analyzers::RegisteredRepository;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SemanticFingerprintOwner {
    JuliaParserSummary,
    ModelicaParserSummary,
    GenericAst(Lang),
}

impl SemanticFingerprintOwner {
    pub(super) fn mode_label(self) -> String {
        match self {
            Self::JuliaParserSummary => "semantic:julia_parser_summary".to_string(),
            Self::ModelicaParserSummary => "semantic:modelica_parser_summary".to_string(),
            Self::GenericAst(lang) => format!("semantic:generic_ast:{}", lang.as_str()),
        }
    }
}

fn plugin_id_supports_semantic_owner_dispatch(plugin_id: &str) -> bool {
    matches!(plugin_id, "julia" | "modelica")
        || Lang::try_from(plugin_id)
            .ok()
            .is_some_and(supports_semantic_fingerprint)
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
    compute_semantic_fingerprint(owner, repository, relative_path, source_text)
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

    if plugin_ids.iter().any(|plugin_id| plugin_id == "julia")
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

    let lang = Lang::from_path(Path::new(relative_path))?;
    supports_semantic_fingerprint(lang).then_some(SemanticFingerprintOwner::GenericAst(lang))
}

pub(super) fn compute_semantic_fingerprint(
    owner: SemanticFingerprintOwner,
    repository: &RegisteredRepository,
    relative_path: &str,
    source_text: &str,
) -> Option<String> {
    match owner {
        SemanticFingerprintOwner::JuliaParserSummary => {
            julia_parser_summary_file_semantic_fingerprint_for_repository(
                repository,
                relative_path,
                source_text,
            )
            .ok()
        }
        SemanticFingerprintOwner::ModelicaParserSummary => {
            modelica_parser_summary_file_semantic_fingerprint_for_repository(
                repository,
                relative_path,
                source_text,
            )
            .ok()
        }
        SemanticFingerprintOwner::GenericAst(lang) => {
            generic_ast_semantic_fingerprint(source_text, lang)
        }
    }
}

fn has_extension(relative_path: &str, extension: &str) -> bool {
    Path::new(relative_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|current| current.eq_ignore_ascii_case(extension))
}
