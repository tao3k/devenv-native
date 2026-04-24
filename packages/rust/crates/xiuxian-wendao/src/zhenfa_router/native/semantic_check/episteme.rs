//! Episteme manifest loading for `wendao audit --load`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_FORBIDDEN_SQL_OPERATIONS: &[&str] = &[
    "CREATE", "ALTER", "DROP", "INSERT", "UPDATE", "DELETE", "MERGE", "COPY", "ATTACH",
];

/// Loaded episteme manifest summary attached to audit results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EpistemeLoadReport {
    /// Manifest display name.
    pub name: Option<String>,
    /// Manifest schema version.
    pub schema_version: Option<u32>,
    /// Canonical manifest path.
    pub manifest_path: String,
    /// Canonical episteme repository root.
    pub root_path: String,
    /// Number of validated policy queries.
    pub policy_query_count: usize,
    /// Number of validated diagnostic mappings.
    pub diagnostic_mapping_count: usize,
    /// Number of validated repair prompts.
    pub repair_prompt_count: usize,
    /// Number of validated repair guards.
    pub repair_guard_count: usize,
    /// Number of validated source-evolution skill surfaces.
    pub source_evolution_skill_count: usize,
    /// Validated policy query summaries.
    pub policy_queries: Vec<EpistemePolicyQueryReport>,
}

/// Validated policy query metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EpistemePolicyQueryReport {
    /// Policy query id.
    pub id: String,
    /// Owning framework id.
    pub framework: Option<String>,
    /// Repository-relative policy query path.
    pub path: String,
    /// Effective statement mode after manifest defaults are applied.
    pub statement_mode: String,
}

#[derive(Debug, Error)]
pub(super) enum EpistemeLoadError {
    #[error("episteme path does not exist: {path}")]
    MissingLoadPath { path: PathBuf },
    #[error("episteme manifest not found: {path}")]
    MissingManifest { path: PathBuf },
    #[error("failed to read episteme manifest {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse episteme manifest {path}: {source}")]
    ParseManifest {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("{section} entry {id} must use a repository-relative path, got {path}")]
    AbsoluteDeclaredPath {
        section: &'static str,
        id: String,
        path: PathBuf,
    },
    #[error("{section} entry {id} references missing file: {path}")]
    MissingDeclaredFile {
        section: &'static str,
        id: String,
        path: PathBuf,
    },
    #[error("{section} entry has an empty id")]
    EmptyId { section: &'static str },
    #[error("duplicate {section} id: {id}")]
    DuplicateId { section: &'static str, id: String },
    #[error("policy query {id} must use statement_mode = \"select_only\", got {statement_mode}")]
    NonSelectStatementMode { id: String, statement_mode: String },
    #[error("policy query {id} contains forbidden SQL operation {operation}")]
    ForbiddenSqlOperation { id: String, operation: String },
    #[error("policy query {id} contains a non-SELECT statement: {statement}")]
    NonSelectStatement { id: String, statement: String },
    #[error("diagnostic mapping {id} references unknown policy query {query}")]
    UnknownDiagnosticQuery { id: String, query: String },
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct EpistemeManifest {
    schema_version: Option<u32>,
    name: Option<String>,
    sql: ManifestSql,
    policy_queries: Vec<ManifestPolicyQuery>,
    diagnostic_mappings: Vec<ManifestDiagnosticMapping>,
    repair_prompts: Vec<ManifestPathEntry>,
    repair_guards: Vec<ManifestPathEntry>,
    source_evolution_skill_surfaces: Vec<ManifestSourceEvolutionSkillSurface>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ManifestSql {
    statement_mode: Option<String>,
    forbidden_operations: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ManifestPolicyQuery {
    id: String,
    framework: Option<String>,
    path: String,
    statement_mode: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ManifestDiagnosticMapping {
    id: String,
    query: Option<String>,
    path: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ManifestPathEntry {
    id: String,
    path: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ManifestSourceEvolutionSkillSurface {
    id: String,
    sources_path: String,
    skill_path: String,
}

pub(super) fn load_episteme_manifest(
    load_path: impl AsRef<Path>,
) -> Result<EpistemeLoadReport, EpistemeLoadError> {
    let (manifest_path, root_path) = resolve_manifest_path(load_path.as_ref())?;
    let manifest_text =
        fs::read_to_string(&manifest_path).map_err(|source| EpistemeLoadError::ReadManifest {
            path: manifest_path.clone(),
            source,
        })?;
    let manifest: EpistemeManifest =
        toml::from_str(&manifest_text).map_err(|source| EpistemeLoadError::ParseManifest {
            path: manifest_path.clone(),
            source,
        })?;

    validate_manifest_ids(&manifest)?;

    let mut policy_queries = Vec::with_capacity(manifest.policy_queries.len());
    let policy_query_ids: BTreeSet<&str> = manifest
        .policy_queries
        .iter()
        .map(|query| query.id.as_str())
        .collect();

    for query in &manifest.policy_queries {
        let query_path = validate_declared_file(
            &root_path,
            "policy_queries",
            &query.id,
            Path::new(&query.path),
        )?;
        let statement_mode = query
            .statement_mode
            .as_deref()
            .or(manifest.sql.statement_mode.as_deref())
            .unwrap_or("select_only");
        validate_statement_mode(&query.id, statement_mode)?;
        validate_policy_sql(&query.id, &query_path, forbidden_operations(&manifest.sql))?;

        policy_queries.push(EpistemePolicyQueryReport {
            id: query.id.clone(),
            framework: query.framework.clone(),
            path: query.path.clone(),
            statement_mode: statement_mode.to_string(),
        });
    }

    for mapping in &manifest.diagnostic_mappings {
        validate_declared_file(
            &root_path,
            "diagnostic_mappings",
            &mapping.id,
            Path::new(&mapping.path),
        )?;
        if let Some(query) = mapping.query.as_deref()
            && !policy_query_ids.contains(query)
        {
            return Err(EpistemeLoadError::UnknownDiagnosticQuery {
                id: mapping.id.clone(),
                query: query.to_string(),
            });
        }
    }

    validate_path_entries(
        &root_path,
        "repair_prompts",
        manifest
            .repair_prompts
            .iter()
            .map(|entry| (&entry.id, &entry.path)),
    )?;
    validate_path_entries(
        &root_path,
        "repair_guards",
        manifest
            .repair_guards
            .iter()
            .map(|entry| (&entry.id, &entry.path)),
    )?;
    validate_source_skill_surfaces(&root_path, &manifest.source_evolution_skill_surfaces)?;

    Ok(EpistemeLoadReport {
        name: manifest.name,
        schema_version: manifest.schema_version,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        root_path: root_path.to_string_lossy().to_string(),
        policy_query_count: policy_queries.len(),
        diagnostic_mapping_count: manifest.diagnostic_mappings.len(),
        repair_prompt_count: manifest.repair_prompts.len(),
        repair_guard_count: manifest.repair_guards.len(),
        source_evolution_skill_count: manifest.source_evolution_skill_surfaces.len(),
        policy_queries,
    })
}

fn resolve_manifest_path(load_path: &Path) -> Result<(PathBuf, PathBuf), EpistemeLoadError> {
    if !load_path.exists() {
        return Err(EpistemeLoadError::MissingLoadPath {
            path: load_path.to_path_buf(),
        });
    }

    let manifest_path = if load_path.is_dir() {
        load_path.join("episteme.toml")
    } else {
        load_path.to_path_buf()
    };

    if !manifest_path.is_file() {
        return Err(EpistemeLoadError::MissingManifest {
            path: manifest_path,
        });
    }

    let manifest_path =
        manifest_path
            .canonicalize()
            .map_err(|source| EpistemeLoadError::ReadManifest {
                path: manifest_path.clone(),
                source,
            })?;
    let root_path = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    Ok((manifest_path, root_path))
}

fn validate_manifest_ids(manifest: &EpistemeManifest) -> Result<(), EpistemeLoadError> {
    validate_unique_ids(
        "policy_queries",
        manifest
            .policy_queries
            .iter()
            .map(|query| query.id.as_str()),
    )?;
    validate_unique_ids(
        "diagnostic_mappings",
        manifest
            .diagnostic_mappings
            .iter()
            .map(|mapping| mapping.id.as_str()),
    )?;
    validate_unique_ids(
        "repair_prompts",
        manifest
            .repair_prompts
            .iter()
            .map(|entry| entry.id.as_str()),
    )?;
    validate_unique_ids(
        "repair_guards",
        manifest.repair_guards.iter().map(|entry| entry.id.as_str()),
    )?;
    validate_unique_ids(
        "source_evolution_skill_surfaces",
        manifest
            .source_evolution_skill_surfaces
            .iter()
            .map(|entry| entry.id.as_str()),
    )
}

fn validate_unique_ids<'a>(
    section: &'static str,
    ids: impl Iterator<Item = &'a str>,
) -> Result<(), EpistemeLoadError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(EpistemeLoadError::EmptyId { section });
        }
        if !seen.insert(trimmed.to_string()) {
            return Err(EpistemeLoadError::DuplicateId {
                section,
                id: trimmed.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_path_entries<'a>(
    root_path: &Path,
    section: &'static str,
    entries: impl Iterator<Item = (&'a String, &'a String)>,
) -> Result<(), EpistemeLoadError> {
    for (id, path) in entries {
        validate_declared_file(root_path, section, id, Path::new(path))?;
    }
    Ok(())
}

fn validate_source_skill_surfaces(
    root_path: &Path,
    surfaces: &[ManifestSourceEvolutionSkillSurface],
) -> Result<(), EpistemeLoadError> {
    for surface in surfaces {
        validate_declared_file(
            root_path,
            "source_evolution_skill_surfaces.sources_path",
            &surface.id,
            Path::new(&surface.sources_path),
        )?;
        validate_declared_file(
            root_path,
            "source_evolution_skill_surfaces.skill_path",
            &surface.id,
            Path::new(&surface.skill_path),
        )?;
    }
    Ok(())
}

fn validate_declared_file(
    root_path: &Path,
    section: &'static str,
    id: &str,
    relative_path: &Path,
) -> Result<PathBuf, EpistemeLoadError> {
    if relative_path.is_absolute() {
        return Err(EpistemeLoadError::AbsoluteDeclaredPath {
            section,
            id: id.to_string(),
            path: relative_path.to_path_buf(),
        });
    }

    let declared_path = root_path.join(relative_path);
    if !declared_path.is_file() {
        return Err(EpistemeLoadError::MissingDeclaredFile {
            section,
            id: id.to_string(),
            path: declared_path,
        });
    }

    Ok(declared_path)
}

fn validate_statement_mode(id: &str, statement_mode: &str) -> Result<(), EpistemeLoadError> {
    if statement_mode == "select_only" {
        return Ok(());
    }
    Err(EpistemeLoadError::NonSelectStatementMode {
        id: id.to_string(),
        statement_mode: statement_mode.to_string(),
    })
}

fn validate_policy_sql(
    id: &str,
    query_path: &Path,
    forbidden_operations: Vec<&str>,
) -> Result<(), EpistemeLoadError> {
    let sql = fs::read_to_string(query_path).map_err(|source| EpistemeLoadError::ReadManifest {
        path: query_path.to_path_buf(),
        source,
    })?;
    let guard_text = sql_guard_text(&sql);

    for operation in forbidden_operations {
        if contains_sql_token(&guard_text, operation) {
            return Err(EpistemeLoadError::ForbiddenSqlOperation {
                id: id.to_string(),
                operation: operation.to_string(),
            });
        }
    }

    for statement in guard_text
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let statement_upper = statement.to_ascii_uppercase();
        if !statement_upper.starts_with("SELECT") && !statement_upper.starts_with("WITH") {
            return Err(EpistemeLoadError::NonSelectStatement {
                id: id.to_string(),
                statement: statement.lines().next().unwrap_or(statement).to_string(),
            });
        }
    }

    Ok(())
}

fn forbidden_operations(sql: &ManifestSql) -> Vec<&str> {
    if sql.forbidden_operations.is_empty() {
        DEFAULT_FORBIDDEN_SQL_OPERATIONS.to_vec()
    } else {
        sql.forbidden_operations
            .iter()
            .map(String::as_str)
            .collect()
    }
}

fn contains_sql_token(sql: &str, token: &str) -> bool {
    let sql = sql.as_bytes();
    let token_upper = token.to_ascii_uppercase();
    let token = token_upper.as_bytes();

    if token.is_empty() || sql.len() < token.len() {
        return false;
    }

    for start in 0..=(sql.len() - token.len()) {
        if !sql[start..start + token.len()].eq_ignore_ascii_case(token) {
            continue;
        }
        let before = start
            .checked_sub(1)
            .and_then(|idx| sql.get(idx))
            .copied()
            .is_some_and(is_sql_identifier_byte);
        let after = sql
            .get(start + token.len())
            .copied()
            .is_some_and(is_sql_identifier_byte);
        if !before && !after {
            return true;
        }
    }

    false
}

fn is_sql_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn sql_guard_text(sql: &str) -> String {
    let mut output = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_block_comment = false;
                output.push(' ');
                output.push(' ');
            } else if ch == '\n' {
                output.push('\n');
            } else {
                output.push(' ');
            }
            continue;
        }

        if in_single_quote {
            if ch == '\'' {
                if chars.peek() == Some(&'\'') {
                    let _ = chars.next();
                    output.push(' ');
                    output.push(' ');
                    continue;
                }
                in_single_quote = false;
            }
            output.push(' ');
            continue;
        }

        if in_double_quote {
            if ch == '"' {
                in_double_quote = false;
            }
            output.push(' ');
            continue;
        }

        if ch == '-' && chars.peek() == Some(&'-') {
            let _ = chars.next();
            in_line_comment = true;
            output.push(' ');
            output.push(' ');
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_block_comment = true;
            output.push(' ');
            output.push(' ');
            continue;
        }

        if ch == '\'' {
            in_single_quote = true;
            output.push(' ');
            continue;
        }

        if ch == '"' {
            in_double_quote = true;
            output.push(' ');
            continue;
        }

        output.push(ch);
    }

    output
}

#[cfg(test)]
#[path = "../../../../tests/unit/semantic_check_tests/episteme.rs"]
mod tests;
