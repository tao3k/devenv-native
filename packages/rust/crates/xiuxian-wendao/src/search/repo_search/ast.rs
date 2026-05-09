#[cfg(test)]
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use walkdir::{DirEntry, WalkDir};
#[cfg(test)]
use xiuxian_ast::extract_items_for_patterns;
use xiuxian_ast::{Lang, extract_items, get_skeleton_patterns};
use xiuxian_git_repo::SyncMode;

use crate::analyzers::resolve_registered_repository_source;
use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig};
use crate::search::SearchPlaneService;
use crate::search::contracts::{SearchHit, StudioNavigationTarget};

enum RepoAstSearchMode {
    Pattern { pattern: String },
    Analysis { search_term: Option<String> },
}

struct RepoAstFile<'a> {
    repo_id: &'a str,
    relative_path: &'a str,
    lang: Lang,
    content: &'a str,
}

struct SearchAccumulator {
    hits: Vec<SearchHit>,
    seen: HashSet<String>,
    limit: usize,
}

struct GenericAstAnalysisMatch<'a> {
    repo_id: &'a str,
    relative_path: &'a str,
    lang: Lang,
    name: &'a str,
    signature: &'a str,
    line_start: usize,
    line_end: usize,
    score: f64,
}

#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct RepoAstAnalysisIndex {
    records: Vec<RepoAstAnalysisRecord>,
    exact_name_index: HashMap<String, Vec<usize>>,
    file_count: usize,
}

#[derive(Debug, Clone)]
#[cfg(test)]
struct RepoAstAnalysisRecord {
    repo_id: String,
    relative_path: String,
    lang: Lang,
    name: String,
    signature: String,
    line_start: usize,
    line_end: usize,
}

#[cfg(test)]
impl RepoAstAnalysisIndex {
    pub(crate) fn file_count(&self) -> usize {
        self.file_count
    }

    pub(crate) fn symbol_count(&self) -> usize {
        self.records.len()
    }

    pub(crate) fn search(&self, search_term: Option<&str>, limit: usize) -> Vec<SearchHit> {
        if limit == 0 {
            return Vec::new();
        }
        if let Some(search_term) = normalized_exact_search_term(search_term)
            && let Some(record_indexes) = self.exact_name_index.get(search_term.as_str())
        {
            return record_indexes
                .iter()
                .take(limit)
                .map(|index| {
                    let record = &self.records[*index];
                    build_generic_ast_analysis_hit(&GenericAstAnalysisMatch {
                        repo_id: record.repo_id.as_str(),
                        relative_path: record.relative_path.as_str(),
                        lang: record.lang,
                        name: record.name.as_str(),
                        signature: record.signature.as_str(),
                        line_start: record.line_start,
                        line_end: record.line_end,
                        score: 1.0,
                    })
                })
                .collect();
        }

        let mut matches = self
            .records
            .iter()
            .filter_map(|record| {
                let score = generic_ast_analysis_score(
                    search_term,
                    record.relative_path.as_str(),
                    record.name.as_str(),
                    record.signature.as_str(),
                )?;
                Some((score, record))
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            right
                .0
                .partial_cmp(&left.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.1.relative_path.cmp(&right.1.relative_path))
                .then_with(|| left.1.line_start.cmp(&right.1.line_start))
        });

        matches
            .into_iter()
            .take(limit)
            .map(|(score, record)| {
                build_generic_ast_analysis_hit(&GenericAstAnalysisMatch {
                    repo_id: record.repo_id.as_str(),
                    relative_path: record.relative_path.as_str(),
                    lang: record.lang,
                    name: record.name.as_str(),
                    signature: record.signature.as_str(),
                    line_start: record.line_start,
                    line_end: record.line_end,
                    score,
                })
            })
            .collect()
    }
}

pub(crate) async fn search_repo_ast_pattern_hits(
    search_plane: &SearchPlaneService,
    repository: &RegisteredRepository,
    pattern: &str,
    language_filters: &[String],
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    let repository = repository.clone();
    let project_root = search_plane.project_root().to_path_buf();
    let language_filters = language_filters.to_vec();
    let mode = RepoAstSearchMode::Pattern {
        pattern: pattern.to_string(),
    };

    tokio::task::spawn_blocking(move || {
        search_repo_ast_hits_blocking(
            project_root.as_path(),
            &repository,
            &mode,
            language_filters.as_slice(),
            limit,
        )
    })
    .await
    .map_err(|error| format!("ast-grep repo search task failed: {error}"))?
}

pub(crate) async fn search_repo_ast_analysis_hits(
    search_plane: &SearchPlaneService,
    repository: &RegisteredRepository,
    search_term: Option<&str>,
    language_filters: &[String],
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    let repository = repository.clone();
    let project_root = search_plane.project_root().to_path_buf();
    let language_filters = language_filters.to_vec();
    let mode = RepoAstSearchMode::Analysis {
        search_term: normalize_analysis_search_term(repository.id.as_str(), search_term),
    };

    tokio::task::spawn_blocking(move || {
        search_repo_ast_hits_blocking(
            project_root.as_path(),
            &repository,
            &mode,
            language_filters.as_slice(),
            limit,
        )
    })
    .await
    .map_err(|error| format!("ast-grep repo search task failed: {error}"))?
}

#[cfg(test)]
pub(crate) fn build_repo_ast_analysis_index_from_checkout(
    checkout_root: &Path,
    repository: &RegisteredRepository,
    language_filters: &[String],
    include_dirs: &[String],
    excluded_dirs: &[String],
) -> RepoAstAnalysisIndex {
    let excluded_languages = excluded_ast_languages_for_repository(repository);
    let normalized_filters = normalized_language_filters(language_filters);
    let mut records = Vec::new();
    let mut seen = HashSet::new();
    let mut file_count = 0;

    for root in repo_ast_index_roots(checkout_root, include_dirs) {
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(root.as_path())
            .into_iter()
            .filter_entry(|entry| {
                should_descend_into_entry(entry)
                    && should_descend_into_catalog_entry(checkout_root, entry, excluded_dirs)
            })
        {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_file() {
                continue;
            }

            let Some(lang) = supported_ast_lang(entry.path(), &excluded_languages) else {
                continue;
            };
            if !normalized_filters.is_empty() && !normalized_filters.contains(lang.as_str()) {
                continue;
            }

            let Ok(content) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            file_count += 1;
            let relative_path = normalize_repo_relative_path(checkout_root, entry.path());
            push_analysis_records(
                &mut records,
                &mut seen,
                repository.id.as_str(),
                relative_path.as_str(),
                lang,
                content.as_str(),
            );
        }
    }

    let exact_name_index = build_exact_name_index(records.as_slice());
    RepoAstAnalysisIndex {
        records,
        exact_name_index,
        file_count,
    }
}

fn normalize_analysis_search_term(repo_id: &str, search_term: Option<&str>) -> Option<String> {
    let normalized = search_term
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    if normalized.eq_ignore_ascii_case(repo_id) {
        return None;
    }

    Some(normalized.to_string())
}

/// Return whether a repository is configured for generic AST analysis.
#[must_use]
pub fn repository_supports_generic_ast_analysis(repository: &RegisteredRepository) -> bool {
    repository
        .plugins
        .iter()
        .any(|plugin| plugin.id().eq_ignore_ascii_case("ast-grep"))
}

/// Resolve the generic AST language for a path in the given repository.
#[must_use]
pub fn repository_generic_ast_lang_for_path(
    repository: &RegisteredRepository,
    path: &Path,
) -> Option<Lang> {
    if !repository_supports_generic_ast_analysis(repository) {
        return None;
    }

    let excluded_languages = excluded_ast_languages_for_repository(repository);
    supported_ast_lang(path, &excluded_languages)
}

pub(crate) fn ast_pattern_requests_generic_analysis(pattern: &str) -> bool {
    matches!(pattern.trim(), "$PATTERN")
}

pub(crate) fn has_generic_ast_language_filters(
    repository: &RegisteredRepository,
    language_filters: &HashSet<String>,
) -> bool {
    if language_filters.is_empty() {
        return false;
    }

    let excluded_languages = excluded_ast_languages_for_repository(repository);
    language_filters.iter().any(|language| {
        Lang::try_from(language.as_str())
            .ok()
            .is_some_and(|lang| !excluded_languages.contains(lang.as_str()))
    })
}

fn search_repo_ast_hits_blocking(
    project_root: &Path,
    repository: &RegisteredRepository,
    mode: &RepoAstSearchMode,
    language_filters: &[String],
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let materialized =
        resolve_registered_repository_source(repository, project_root, SyncMode::Ensure).map_err(
            |error| format!("failed to resolve repository `{}`: {error}", repository.id),
        )?;
    let checkout_root = materialized.checkout_root;
    let excluded_languages = excluded_ast_languages_for_repository(repository);
    let normalized_filters = normalized_language_filters(language_filters);
    let mut accumulator = SearchAccumulator::new(limit);

    for entry in WalkDir::new(checkout_root.as_path())
        .into_iter()
        .filter_entry(should_descend_into_entry)
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }

        let Some(lang) = supported_ast_lang(entry.path(), &excluded_languages) else {
            continue;
        };
        if !normalized_filters.is_empty() && !normalized_filters.contains(lang.as_str()) {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let relative_path = normalize_repo_relative_path(checkout_root.as_path(), entry.path());
        let file = RepoAstFile {
            repo_id: repository.id.as_str(),
            relative_path: relative_path.as_str(),
            lang,
            content: content.as_str(),
        };

        let limit_reached = match mode {
            RepoAstSearchMode::Pattern { pattern } => {
                accumulator.push_pattern_matches(&file, pattern.as_str())
            }
            RepoAstSearchMode::Analysis { search_term } => {
                accumulator.push_analysis_matches(&file, search_term.as_deref())
            }
        };

        if limit_reached {
            break;
        }
    }

    Ok(accumulator.finish())
}

fn build_ast_search_hit(
    repo_id: &str,
    relative_path: &str,
    lang: Lang,
    result: &xiuxian_ast::ExtractResult,
) -> SearchHit {
    let name = result
        .captures
        .get("NAME")
        .cloned()
        .or_else(|| {
            Path::new(relative_path)
                .file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| relative_path.to_string());
    let summary = summarize_match_text(result.text.as_str());

    SearchHit {
        stem: name,
        title: Some(relative_path.to_string()),
        path: relative_path.to_string(),
        doc_type: Some("ast_match".to_string()),
        tags: vec![
            repo_id.to_string(),
            "code".to_string(),
            "ast-grep".to_string(),
            "kind:ast_match".to_string(),
            lang.as_str().to_string(),
            format!("lang:{}", lang.as_str()),
        ],
        score: 1.0,
        best_section: Some(summary),
        match_reason: Some("ast-grep structural match".to_string()),
        hierarchical_uri: None,
        hierarchy: Some(relative_path.split('/').map(str::to_string).collect()),
        saliency_score: None,
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target: Some(StudioNavigationTarget {
            path: format!("{repo_id}/{relative_path}"),
            category: "repo_code".to_string(),
            project_name: Some(repo_id.to_string()),
            root_label: Some(repo_id.to_string()),
            line: Some(result.line_start),
            line_end: Some(result.line_end),
            column: None,
        }),
    }
}

fn normalized_language_filters(language_filters: &[String]) -> HashSet<String> {
    language_filters
        .iter()
        .map(|filter| filter.trim().to_ascii_lowercase())
        .filter(|filter| !filter.is_empty())
        .collect()
}

impl SearchAccumulator {
    fn new(limit: usize) -> Self {
        Self {
            hits: Vec::new(),
            seen: HashSet::new(),
            limit,
        }
    }

    fn finish(self) -> Vec<SearchHit> {
        self.hits
    }

    fn push_pattern_matches(&mut self, file: &RepoAstFile<'_>, pattern: &str) -> bool {
        for result in extract_items(file.content, pattern, file.lang, None) {
            let dedupe_key = format!(
                "{}:{}:{}:{}",
                file.relative_path, result.line_start, result.line_end, result.text
            );
            if !self.seen.insert(dedupe_key) {
                continue;
            }

            self.hits.push(build_ast_search_hit(
                file.repo_id,
                file.relative_path,
                file.lang,
                &result,
            ));
            if self.hits.len() >= self.limit {
                return true;
            }
        }

        false
    }

    fn push_analysis_matches(&mut self, file: &RepoAstFile<'_>, search_term: Option<&str>) -> bool {
        for pattern in get_skeleton_patterns(file.lang) {
            for result in extract_items(file.content, pattern, file.lang, Some(vec!["NAME"])) {
                let signature = first_signature_line(result.text.as_str()).to_string();
                if signature.is_empty() {
                    continue;
                }
                let name = result
                    .captures
                    .get("NAME")
                    .cloned()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| signature.clone());
                let Some(score) = generic_ast_analysis_score(
                    search_term,
                    file.relative_path,
                    name.as_str(),
                    signature.as_str(),
                ) else {
                    continue;
                };
                let dedupe_key = format!(
                    "{}:{}:{}:{}",
                    file.relative_path, result.line_start, result.line_end, name
                );
                if !self.seen.insert(dedupe_key) {
                    continue;
                }

                let analysis_match = GenericAstAnalysisMatch {
                    repo_id: file.repo_id,
                    relative_path: file.relative_path,
                    lang: file.lang,
                    name: name.as_str(),
                    signature: signature.as_str(),
                    line_start: result.line_start,
                    line_end: result.line_end,
                    score,
                };
                self.hits
                    .push(build_generic_ast_analysis_hit(&analysis_match));
                if self.hits.len() >= self.limit {
                    return true;
                }
            }
        }

        false
    }
}

fn supported_ast_lang(path: &Path, excluded_languages: &HashSet<String>) -> Option<Lang> {
    let lang = Lang::from_path(path)?;
    (!excluded_languages.contains(lang.as_str())).then_some(lang)
}

fn normalize_repo_relative_path(checkout_root: &Path, path: &Path) -> String {
    path.strip_prefix(checkout_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn excluded_ast_languages_for_repository(repository: &RegisteredRepository) -> HashSet<String> {
    repository
        .plugins
        .iter()
        .flat_map(plugin_ast_excluded_languages)
        .collect()
}

fn plugin_ast_excluded_languages(plugin: &RepositoryPluginConfig) -> Vec<String> {
    let mut languages = Vec::new();
    push_normalized_ast_language(&mut languages, plugin.id());

    if let RepositoryPluginConfig::Config { options, .. } = plugin {
        collect_normalized_ast_languages_from_option(options.get("language"), &mut languages);
        collect_normalized_ast_languages_from_option(options.get("languages"), &mut languages);
        collect_normalized_ast_languages_from_option(
            options.get("ast_grep_exclude_languages"),
            &mut languages,
        );
    }

    languages
}

fn collect_normalized_ast_languages_from_option(
    value: Option<&serde_json::Value>,
    languages: &mut Vec<String>,
) {
    let Some(value) = value else {
        return;
    };

    match value {
        serde_json::Value::String(language) => push_normalized_ast_language(languages, language),
        serde_json::Value::Array(values) => {
            for value in values {
                if let Some(language) = value.as_str() {
                    push_normalized_ast_language(languages, language);
                }
            }
        }
        _ => {}
    }
}

fn push_normalized_ast_language(languages: &mut Vec<String>, language: &str) {
    let trimmed = language.trim();
    if trimmed.is_empty() {
        return;
    }

    let normalized = match Lang::try_from(trimmed) {
        Ok(lang) => lang.as_str().to_string(),
        Err(_) => trimmed.to_ascii_lowercase(),
    };
    languages.push(normalized);
}

#[cfg(test)]
fn repo_ast_index_roots(checkout_root: &Path, include_dirs: &[String]) -> Vec<PathBuf> {
    if include_dirs.is_empty() {
        return vec![checkout_root.to_path_buf()];
    }

    include_dirs
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .map(|path| checkout_root.join(path))
        .collect()
}

#[cfg(test)]
fn should_descend_into_catalog_entry(
    checkout_root: &Path,
    entry: &DirEntry,
    excluded_dirs: &[String],
) -> bool {
    if entry.depth() == 0 || excluded_dirs.is_empty() {
        return true;
    }

    let relative_path = normalize_repo_relative_path(checkout_root, entry.path());
    !excluded_dirs.iter().any(|excluded| {
        let excluded = excluded.trim().trim_matches('/');
        !excluded.is_empty()
            && (relative_path == excluded || relative_path.starts_with(&format!("{excluded}/")))
    })
}

fn first_signature_line(text: &str) -> &str {
    text.lines().next().map(str::trim).unwrap_or_default()
}

fn generic_ast_analysis_score(
    search_term: Option<&str>,
    relative_path: &str,
    name: &str,
    signature: &str,
) -> Option<f64> {
    let Some(search_term) = search_term
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
    else {
        return Some(0.72);
    };
    let normalized_name = name.to_ascii_lowercase();
    let normalized_signature = signature.to_ascii_lowercase();
    let normalized_path = relative_path.to_ascii_lowercase();

    if normalized_name == search_term {
        return Some(1.0);
    }
    if normalized_name.contains(search_term.as_str()) {
        return Some(0.97);
    }
    if normalized_signature.contains(search_term.as_str()) {
        return Some(0.91);
    }
    if normalized_path.contains(search_term.as_str()) {
        return Some(0.84);
    }

    None
}

#[cfg(test)]
fn normalized_exact_search_term(search_term: Option<&str>) -> Option<String> {
    search_term
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
fn build_exact_name_index(records: &[RepoAstAnalysisRecord]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (record_index, record) in records.iter().enumerate() {
        index
            .entry(record.name.to_ascii_lowercase())
            .or_default()
            .push(record_index);
    }
    index
}

fn build_generic_ast_analysis_hit(result: &GenericAstAnalysisMatch<'_>) -> SearchHit {
    SearchHit {
        stem: result.name.to_string(),
        title: Some(result.relative_path.to_string()),
        path: result.relative_path.to_string(),
        doc_type: Some("ast_match".to_string()),
        tags: vec![
            result.repo_id.to_string(),
            "code".to_string(),
            "ast-grep".to_string(),
            "kind:ast_match".to_string(),
            result.lang.as_str().to_string(),
            format!("lang:{}", result.lang.as_str()),
        ],
        score: result.score,
        best_section: Some(result.signature.to_string()),
        match_reason: Some("ast-grep structural analysis".to_string()),
        hierarchical_uri: None,
        hierarchy: Some(
            result
                .relative_path
                .split('/')
                .map(str::to_string)
                .collect(),
        ),
        saliency_score: None,
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target: Some(StudioNavigationTarget {
            path: format!("{}/{}", result.repo_id, result.relative_path),
            category: "repo_code".to_string(),
            project_name: Some(result.repo_id.to_string()),
            root_label: Some(result.repo_id.to_string()),
            line: Some(result.line_start),
            line_end: Some(result.line_end),
            column: None,
        }),
    }
}

#[cfg(test)]
fn push_analysis_records(
    records: &mut Vec<RepoAstAnalysisRecord>,
    seen: &mut HashSet<String>,
    repo_id: &str,
    relative_path: &str,
    lang: Lang,
    content: &str,
) {
    for result in extract_items_for_patterns(
        content,
        get_skeleton_patterns(lang),
        lang,
        Some(vec!["NAME"]),
    ) {
        let signature = first_signature_line(result.text.as_str()).to_string();
        if signature.is_empty() {
            continue;
        }
        let name = result
            .captures
            .get("NAME")
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| signature.clone());
        let dedupe_key = format!(
            "{relative_path}:{}:{}:{name}",
            result.line_start, result.line_end
        );
        if !seen.insert(dedupe_key) {
            continue;
        }
        records.push(RepoAstAnalysisRecord {
            repo_id: repo_id.to_string(),
            relative_path: relative_path.to_string(),
            lang,
            name,
            signature,
            line_start: result.line_start,
            line_end: result.line_end,
        });
    }
}

fn should_descend_into_entry(entry: &DirEntry) -> bool {
    if entry.depth() == 0 || !entry.file_type().is_dir() {
        return true;
    }

    let Some(name) = entry.file_name().to_str() else {
        return false;
    };
    !matches!(
        name,
        ".git" | ".jj" | ".svn" | ".hg" | ".direnv" | "target" | "node_modules"
    )
}

fn summarize_match_text(text: &str) -> String {
    const SUMMARY_LIMIT: usize = 160;
    let summary = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(text)
        .trim();
    if summary.chars().count() <= SUMMARY_LIMIT {
        return summary.to_string();
    }

    summary.chars().take(SUMMARY_LIMIT - 3).collect::<String>() + "..."
}

#[cfg(test)]
#[path = "../../../tests/unit/search/repo_search/ast.rs"]
mod tests;
