//! Runner for semantic check scenario tests.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use walkdir::WalkDir;
use xiuxian_wendao::parsers::markdown::{CodeObservation, is_supported_note, parse_note};
use xiuxian_wendao::zhenfa_router::native::audit::{
    CodeLanguageId, SourceFile, resolve_source_files, suggest_pattern_fix_with_threshold,
};
use xiuxian_wendao::zhenfa_router::native::semantic_check::{FuzzySuggestionData, SemanticIssue};

use crate::scenarios::support::{Scenario, ScenarioRunner};

/// Runner for `semantic_check` category scenarios.
pub struct SemanticCheckRunner;

struct IssueBuildContext<'a> {
    source_files: &'a [SourceFile],
    scenario_dir: &'a Path,
    temp_dir: &'a Path,
}

impl ScenarioRunner for SemanticCheckRunner {
    fn category(&self) -> &'static str {
        "semantic_check"
    }

    fn run(&self, scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>> {
        let source_paths = load_source_paths(scenario, temp_dir)?;
        let docs = parse_scenario_notes(temp_dir)?;
        let docs_checked = sorted_strings(docs.iter().map(|doc| doc.doc_id.clone()).collect());
        let mut issues = Vec::new();

        for doc in &docs {
            for (node_id, observation) in &doc.observations {
                let language_id = CodeLanguageId::from(observation.language.as_str());
                if language_id.as_str() == "unknown" {
                    continue;
                }
                let source_files = resolve_scenario_source_files(&source_paths, &language_id);
                let issue_context = IssueBuildContext {
                    source_files: &source_files,
                    scenario_dir: &scenario.dir,
                    temp_dir,
                };

                if let Err(error) = observation.validate_pattern() {
                    if error.to_string().contains("retired") {
                        continue;
                    }
                    issues.push(build_issue(
                        "error",
                        "invalid_observation_pattern",
                        &doc.doc_id,
                        node_id,
                        observation,
                        &issue_context,
                    ));
                    continue;
                }

                if source_files.is_empty()
                    || count_observation_matches(observation, &language_id, &source_files) > 0
                {
                    continue;
                }

                issues.push(build_issue(
                    "warning",
                    "observation_target_missing",
                    &doc.doc_id,
                    node_id,
                    observation,
                    &issue_context,
                ));
            }
        }

        issues.sort_by(|left, right| {
            left.doc
                .cmp(&right.doc)
                .then_with(|| left.node_id.cmp(&right.node_id))
                .then_with(|| left.issue_type.cmp(&right.issue_type))
                .then_with(|| left.message.cmp(&right.message))
        });

        Ok(json!({
            "scenario_id": scenario.id(),
            "category": scenario.category(),
            "docs_checked": docs_checked,
            "issue_count": issues.len(),
            "issues": issues
                .iter()
                .map(|issue| snapshot_issue(issue, &scenario.dir, temp_dir))
                .collect::<Vec<_>>(),
        }))
    }
}

#[derive(Debug)]
struct ScenarioObservationDoc {
    doc_id: String,
    observations: Vec<(String, CodeObservation)>,
}

fn load_source_paths(scenario: &Scenario, temp_dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let scenario_toml = fs::read_to_string(scenario.dir.join("scenario.toml"))?;
    let config: toml::Value = toml::from_str(&scenario_toml)?;
    let Some(paths) = config
        .get("runner")
        .and_then(|runner| runner.get("source_paths"))
        .and_then(toml::Value::as_array)
    else {
        return Ok(Vec::new());
    };

    let resolved = paths
        .iter()
        .filter_map(toml::Value::as_str)
        .map(|raw| resolve_source_path(scenario, temp_dir, raw))
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    Ok(resolved)
}

fn resolve_source_path(scenario: &Scenario, temp_dir: &Path, raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    let candidates = [
        scenario.dir.join(trimmed),
        scenario.dir.join(trimmed.trim_start_matches("./")),
        scenario.dir.join(trimmed.trim_start_matches("../")),
        temp_dir.join(trimmed),
        temp_dir.join(trimmed.trim_start_matches("./")),
        temp_dir.join(trimmed.trim_start_matches("../")),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return candidate;
        }
    }

    scenario.dir.join(trimmed)
}

fn parse_scenario_notes(temp_dir: &Path) -> Result<Vec<ScenarioObservationDoc>, Box<dyn Error>> {
    let mut docs = Vec::new();

    for entry in WalkDir::new(temp_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !entry.file_type().is_file() || !is_supported_note(path) {
            continue;
        }

        let content = fs::read_to_string(path)?;
        let Some(parsed) = parse_note(path, temp_dir, &content) else {
            continue;
        };
        let doc_id = parsed.doc.id.clone();

        let observations = parsed
            .sections
            .iter()
            .flat_map(|section| {
                let doc_id = doc_id.clone();
                section.observations.iter().map(move |observation| {
                    let node_id = if section.heading_path.is_empty() {
                        doc_id.clone()
                    } else {
                        format!("{doc_id}#{}", section.heading_path.replace(" / ", "/"))
                    };
                    (node_id, observation.clone())
                })
            })
            .collect::<Vec<_>>();

        docs.push(ScenarioObservationDoc {
            doc_id,
            observations,
        });
    }

    Ok(docs)
}

fn resolve_scenario_source_files(
    paths: &[String],
    language_id: &CodeLanguageId,
) -> Vec<SourceFile> {
    let path_refs = paths.iter().map(Path::new).collect::<Vec<_>>();
    resolve_source_files(&path_refs, language_id)
}

fn count_observation_matches(
    observation: &CodeObservation,
    language_id: &CodeLanguageId,
    source_files: &[SourceFile],
) -> usize {
    source_files
        .iter()
        .filter(|file| observation.matches_scope(&file.path))
        .filter_map(|file| {
            count_code_pattern_matches_for_language_id(
                &file.content,
                &observation.pattern,
                language_id,
            )
            .ok()
        })
        .sum()
}

fn count_code_pattern_matches_for_language_id(
    content: &str,
    pattern: &str,
    _language_id: &CodeLanguageId,
) -> Result<usize, String> {
    let tokens = pattern
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| token.len() > 2 && !token.starts_with('$'))
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return Ok(0);
    }
    Ok(tokens
        .iter()
        .filter(|token| content.contains(*token))
        .count())
}

fn build_issue(
    severity: &str,
    issue_type: &str,
    doc_id: &str,
    node_id: &str,
    observation: &CodeObservation,
    context: &IssueBuildContext<'_>,
) -> SemanticIssue {
    let language_id = CodeLanguageId::from(observation.language.as_str());
    let fuzzy_suggestion = (language_id.as_str() != "unknown")
        .then(|| {
            suggest_pattern_fix_with_threshold(
                &observation.pattern,
                &language_id,
                context.source_files,
                Some(0.65),
            )
        })
        .flatten()
        .map(|suggestion| FuzzySuggestionData {
            original_pattern: observation.pattern.clone(),
            suggested_pattern: suggestion.suggested_pattern,
            confidence: suggestion.confidence,
            source_location: suggestion.source_location.map(|value| {
                sanitize_source_location(&value, context.scenario_dir, context.temp_dir)
            }),
            replacement_drawer: suggestion.replacement_drawer,
        });

    let message = match issue_type {
        "invalid_observation_pattern" => {
            format!(
                "Invalid sgrep pattern in :OBSERVE:: {}",
                observation.pattern
            )
        }
        "observation_target_missing" => format!(
            "Observation pattern '{}' found no matches in source files",
            observation.pattern
        ),
        _ => observation.pattern.clone(),
    };

    SemanticIssue {
        severity: severity.to_string(),
        issue_type: issue_type.to_string(),
        doc: doc_id.to_string(),
        node_id: node_id.to_string(),
        message,
        location: None,
        suggestion: None,
        fuzzy_suggestion,
    }
}

fn snapshot_issue(
    issue: &xiuxian_wendao::zhenfa_router::native::semantic_check::SemanticIssue,
    scenario_dir: &Path,
    temp_dir: &Path,
) -> Value {
    json!({
        "severity": issue.severity,
        "issue_type": issue.issue_type,
        "doc": sanitize_path(&issue.doc, scenario_dir, temp_dir),
        "node_id": issue.node_id,
        "message": issue.message,
        "has_fuzzy_suggestion": issue.fuzzy_suggestion.is_some(),
        "fuzzy_suggestion": issue.fuzzy_suggestion.as_ref().map(|fuzzy| json!({
            "suggested_pattern": fuzzy.suggested_pattern,
            "confidence": round_confidence(fuzzy.confidence),
            "source_location": fuzzy
                .source_location
                .as_ref()
                .map(|value| sanitize_source_location(value, scenario_dir, temp_dir)),
            "replacement_drawer": fuzzy.replacement_drawer,
        })),
    })
}

fn sanitize_source_location(location: &str, scenario_dir: &Path, temp_dir: &Path) -> String {
    let Some((path, line)) = location.rsplit_once(':') else {
        return sanitize_path(location, scenario_dir, temp_dir);
    };
    if line.parse::<usize>().is_err() {
        return sanitize_path(location, scenario_dir, temp_dir);
    }
    format!("{}:{line}", sanitize_path(path, scenario_dir, temp_dir))
}

fn sanitize_path(path: &str, scenario_dir: &Path, temp_dir: &Path) -> String {
    let candidate = Path::new(path);
    if let Ok(stripped) = candidate.strip_prefix(temp_dir) {
        return display_relative_path(stripped);
    }
    if let Ok(stripped) = candidate.strip_prefix(scenario_dir) {
        return display_relative_path(stripped);
    }
    display_relative_path(candidate)
}

fn display_relative_path(path: &Path) -> String {
    let rendered = path.display().to_string().replace('\\', "/");
    rendered.trim_start_matches("./").to_string()
}

fn round_confidence(confidence: f32) -> f64 {
    (f64::from(confidence) * 100.0).round() / 100.0
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}
