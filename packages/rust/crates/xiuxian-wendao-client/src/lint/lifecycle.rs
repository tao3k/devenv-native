//! Semantic lifecycle preview and explicit writeback helpers.

use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use xiuxian_wendao_parsers::semantic_ssot::{
    SemanticChangeIntent, SemanticObject, SemanticRepository, SemanticStatusTransition,
};
use xiuxian_wendao_parsers::{
    SemanticProjectionStaleness, SemanticStatus, SemanticValidationIssue, load_semantic_repository,
    split_frontmatter_raw,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SemanticLifecyclePlanReport {
    pub(crate) entry_count: usize,
    pub(crate) promotion_count: usize,
    pub(crate) demotion_count: usize,
    pub(crate) other_transition_count: usize,
    pub(crate) pending_apply_count: usize,
    pub(crate) already_applied_count: usize,
    pub(crate) blocked_count: usize,
    pub(crate) entries: Vec<SemanticLifecyclePlanEntry>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SemanticLifecyclePlanEntry {
    pub(crate) change_intent_id: String,
    pub(crate) object_id: String,
    pub(crate) current: Option<String>,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) outcome: String,
    pub(crate) writeback_action: String,
}

pub(crate) fn apply_semantic_lifecycle_plan(root: &Path) -> Result<usize> {
    let repository = load_semantic_repository(root);
    let plan = semantic_lifecycle_plan_report(&repository);
    ensure_lifecycle_plan_applyable(root, &repository, &plan)?;
    if plan.pending_apply_count == 0 {
        return Ok(0);
    }

    let pending_entries = plan
        .entries
        .iter()
        .filter(|entry| entry.writeback_action == "pending_apply")
        .collect::<Vec<_>>();
    ensure_unique_lifecycle_apply_targets(pending_entries.as_slice())?;

    let object_by_id = repository
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();
    let mut applied_object_ids = BTreeSet::new();
    let mut promoted_object_ids = BTreeSet::new();
    let mut applied_count = 0usize;

    for entry in pending_entries {
        let object = object_by_id
            .get(entry.object_id.as_str())
            .with_context(|| {
                format!(
                    "semantic lifecycle apply target `{}` does not resolve",
                    entry.object_id
                )
            })?;
        apply_object_lifecycle_writeback(
            root,
            object,
            entry.to.as_str(),
            entry.outcome == "promotion",
        )?;
        applied_object_ids.insert(entry.object_id.clone());
        if entry.outcome == "promotion" {
            promoted_object_ids.insert(entry.object_id.clone());
        }
        applied_count += 1;
    }

    if !promoted_object_ids.is_empty() {
        remove_promoted_candidate_suggestions(root, &repository, &promoted_object_ids)?;
    }
    mark_lifecycle_projection_sources_stale(root, &repository, &applied_object_ids)?;

    Ok(applied_count)
}

pub(crate) fn semantic_lifecycle_plan_report(
    repository: &SemanticRepository,
) -> SemanticLifecyclePlanReport {
    let object_by_id = repository
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();
    let entries = repository
        .change_intents
        .iter()
        .filter(|intent| intent.status == SemanticStatus::Active)
        .flat_map(|intent| semantic_lifecycle_plan_entries(intent, &object_by_id))
        .collect::<Vec<_>>();
    let promotion_count = entries
        .iter()
        .filter(|entry| entry.outcome == "promotion")
        .count();
    let demotion_count = entries
        .iter()
        .filter(|entry| entry.outcome == "demotion")
        .count();
    let already_applied_count = entries
        .iter()
        .filter(|entry| entry.writeback_action == "already_applied")
        .count();
    let pending_apply_count = entries
        .iter()
        .filter(|entry| entry.writeback_action == "pending_apply")
        .count();
    let blocked_count = entries
        .iter()
        .filter(|entry| entry.writeback_action.starts_with("blocked"))
        .count();
    SemanticLifecyclePlanReport {
        entry_count: entries.len(),
        promotion_count,
        demotion_count,
        other_transition_count: entries
            .len()
            .saturating_sub(promotion_count)
            .saturating_sub(demotion_count),
        pending_apply_count,
        already_applied_count,
        blocked_count,
        entries,
    }
}

fn ensure_lifecycle_plan_applyable(
    root: &Path,
    repository: &SemanticRepository,
    plan: &SemanticLifecyclePlanReport,
) -> Result<()> {
    let allowed_pending_messages = plan
        .entries
        .iter()
        .filter(|entry| entry.writeback_action == "pending_apply")
        .map(|entry| {
            format!(
                "semantic status transition `{}` current status must match transition target",
                entry.object_id
            )
        })
        .collect::<BTreeSet<_>>();
    let blocking_issues = repository
        .report
        .issues
        .iter()
        .filter(|issue| !allowed_pending_messages.contains(issue.message.as_str()))
        .collect::<Vec<_>>();
    if !blocking_issues.is_empty() {
        bail!(
            "semantic lifecycle apply has blocking validation issue(s): {}",
            render_semantic_validation_issues(root, blocking_issues.as_slice())
        );
    }
    if plan.blocked_count == 0 {
        return Ok(());
    }

    let blocked_entries = plan
        .entries
        .iter()
        .filter(|entry| entry.writeback_action.starts_with("blocked"))
        .map(|entry| {
            format!(
                "{} current={} expected_from={} target={}",
                entry.object_id,
                entry.current.as_deref().unwrap_or("<missing>"),
                entry.from,
                entry.to
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    bail!("semantic lifecycle apply has blocked transition target(s): {blocked_entries}");
}

fn render_semantic_validation_issues(root: &Path, issues: &[&SemanticValidationIssue]) -> String {
    issues
        .iter()
        .map(|issue| {
            let path = issue.path.as_ref().map_or_else(
                || root.display().to_string(),
                |path| root.join(path).display().to_string(),
            );
            format!("{path}: {}", issue.message)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn ensure_unique_lifecycle_apply_targets(entries: &[&SemanticLifecyclePlanEntry]) -> Result<()> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for entry in entries {
        if !seen.insert(entry.object_id.as_str()) {
            duplicates.insert(entry.object_id.as_str());
        }
    }
    if duplicates.is_empty() {
        return Ok(());
    }
    bail!(
        "semantic lifecycle apply has duplicate pending target(s): {}",
        duplicates.into_iter().collect::<Vec<_>>().join(", ")
    );
}

fn apply_object_lifecycle_writeback(
    root: &Path,
    object: &SemanticObject,
    target_status: &str,
    is_promotion: bool,
) -> Result<()> {
    let object_path = root.join(&object.source_path);
    let content = std::fs::read_to_string(object_path.as_path())
        .with_context(|| format!("failed to read semantic object `{}`", object_path.display()))?;
    let parts = split_frontmatter_raw(&content).with_context(|| {
        format!(
            "semantic object `{}` is missing frontmatter",
            object_path.display()
        )
    })?;
    let mut frontmatter =
        serde_yaml::from_str::<serde_yaml::Value>(parts.yaml).with_context(|| {
            format!(
                "failed to parse semantic object frontmatter `{}`",
                object_path.display()
            )
        })?;
    update_object_lifecycle_frontmatter(&mut frontmatter, target_status, is_promotion)?;
    let rendered = render_semantic_document(&frontmatter, parts.body)?;
    std::fs::write(object_path.as_path(), rendered).with_context(|| {
        format!(
            "failed to write semantic object `{}`",
            object_path.display()
        )
    })?;
    Ok(())
}

fn update_object_lifecycle_frontmatter(
    frontmatter: &mut serde_yaml::Value,
    target_status: &str,
    is_promotion: bool,
) -> Result<()> {
    let Some(mapping) = frontmatter.as_mapping_mut() else {
        bail!("semantic object frontmatter must be a YAML mapping");
    };
    mapping.insert(
        serde_yaml::Value::String("status".to_string()),
        serde_yaml::Value::String(target_status.to_string()),
    );
    if !is_promotion {
        return Ok(());
    }

    let confidence_key = serde_yaml::Value::String("confidence".to_string());
    let Some(confidence) = mapping
        .get_mut(&confidence_key)
        .and_then(serde_yaml::Value::as_mapping_mut)
    else {
        bail!("semantic object promotion requires confidence mapping");
    };
    confidence.insert(
        serde_yaml::Value::String("source".to_string()),
        serde_yaml::Value::String("human_signed".to_string()),
    );
    Ok(())
}

fn remove_promoted_candidate_suggestions(
    root: &Path,
    repository: &SemanticRepository,
    promoted_object_ids: &BTreeSet<String>,
) -> Result<usize> {
    let mut changed_file_count = 0usize;
    for intent in &repository.change_intents {
        if !intent
            .candidate_suggestions
            .iter()
            .any(|object_id| promoted_object_ids.contains(object_id))
        {
            continue;
        }
        let intent_path = root.join(&intent.source_path);
        let content = std::fs::read_to_string(intent_path.as_path()).with_context(|| {
            format!(
                "failed to read semantic change intent `{}`",
                intent_path.display()
            )
        })?;
        let parts = split_frontmatter_raw(&content).with_context(|| {
            format!(
                "semantic change intent `{}` is missing frontmatter",
                intent_path.display()
            )
        })?;
        let mut frontmatter =
            serde_yaml::from_str::<serde_yaml::Value>(parts.yaml).with_context(|| {
                format!(
                    "failed to parse semantic change intent frontmatter `{}`",
                    intent_path.display()
                )
            })?;
        if remove_candidate_suggestions_from_frontmatter(&mut frontmatter, promoted_object_ids)? {
            let rendered = render_semantic_document(&frontmatter, parts.body)?;
            std::fs::write(intent_path.as_path(), rendered).with_context(|| {
                format!(
                    "failed to write semantic change intent `{}`",
                    intent_path.display()
                )
            })?;
            changed_file_count += 1;
        }
    }
    Ok(changed_file_count)
}

fn remove_candidate_suggestions_from_frontmatter(
    frontmatter: &mut serde_yaml::Value,
    promoted_object_ids: &BTreeSet<String>,
) -> Result<bool> {
    let Some(mapping) = frontmatter.as_mapping_mut() else {
        bail!("semantic change intent frontmatter must be a YAML mapping");
    };
    let key = serde_yaml::Value::String("candidate_suggestions".to_string());
    let Some(value) = mapping.get_mut(&key) else {
        return Ok(false);
    };
    let Some(sequence) = value.as_sequence_mut() else {
        bail!("semantic change intent candidate_suggestions must be a YAML sequence");
    };
    let original_len = sequence.len();
    sequence.retain(|value| {
        value
            .as_str()
            .is_none_or(|object_id| !promoted_object_ids.contains(object_id))
    });
    Ok(sequence.len() != original_len)
}

fn mark_lifecycle_projection_sources_stale(
    root: &Path,
    repository: &SemanticRepository,
    applied_object_ids: &BTreeSet<String>,
) -> Result<usize> {
    let mut changed_count = 0usize;
    for projection in &repository.projections {
        if projection.staleness == SemanticProjectionStaleness::Stale
            || !projection
                .source_objects
                .iter()
                .any(|object_id| applied_object_ids.contains(object_id))
        {
            continue;
        }
        let projection_path = root.join(&projection.source_path);
        let content = std::fs::read_to_string(projection_path.as_path()).with_context(|| {
            format!(
                "failed to read semantic projection `{}`",
                projection_path.display()
            )
        })?;
        let parts = split_frontmatter_raw(&content).with_context(|| {
            format!(
                "semantic projection `{}` is missing frontmatter",
                projection_path.display()
            )
        })?;
        let mut frontmatter =
            serde_yaml::from_str::<serde_yaml::Value>(parts.yaml).with_context(|| {
                format!(
                    "failed to parse semantic projection frontmatter `{}`",
                    projection_path.display()
                )
            })?;
        mark_projection_frontmatter_stale(&mut frontmatter, projection.source_revision.as_str())?;
        let rendered = render_semantic_document(&frontmatter, parts.body)?;
        std::fs::write(projection_path.as_path(), rendered).with_context(|| {
            format!(
                "failed to write semantic projection `{}`",
                projection_path.display()
            )
        })?;
        changed_count += 1;
    }
    Ok(changed_count)
}

fn mark_projection_frontmatter_stale(
    frontmatter: &mut serde_yaml::Value,
    source_revision: &str,
) -> Result<()> {
    let Some(mapping) = frontmatter.as_mapping_mut() else {
        bail!("semantic projection frontmatter must be a YAML mapping");
    };
    mapping.insert(
        serde_yaml::Value::String("source_revision".to_string()),
        serde_yaml::Value::String(source_revision.to_string()),
    );
    mapping.insert(
        serde_yaml::Value::String("staleness".to_string()),
        serde_yaml::Value::String("stale".to_string()),
    );
    Ok(())
}

fn semantic_lifecycle_plan_entries(
    intent: &SemanticChangeIntent,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
) -> Vec<SemanticLifecyclePlanEntry> {
    let promotion_targets = intent
        .promotion_targets
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let demotion_targets = intent
        .demotion_targets
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    intent
        .status_transitions
        .iter()
        .map(|transition| {
            semantic_lifecycle_plan_entry(
                intent,
                transition,
                object_by_id,
                &promotion_targets,
                &demotion_targets,
            )
        })
        .collect()
}

fn semantic_lifecycle_plan_entry(
    intent: &SemanticChangeIntent,
    transition: &SemanticStatusTransition,
    object_by_id: &BTreeMap<&str, &SemanticObject>,
    promotion_targets: &BTreeSet<&str>,
    demotion_targets: &BTreeSet<&str>,
) -> SemanticLifecyclePlanEntry {
    let object_id = transition.object_id.as_str();
    let outcome = if promotion_targets.contains(object_id) {
        "promotion"
    } else if demotion_targets.contains(object_id) {
        "demotion"
    } else {
        "status_transition"
    };
    let current = object_by_id
        .get(object_id)
        .map(|object| semantic_status_token(&object.status).to_string());
    let from = semantic_status_token(&transition.from);
    let to = semantic_status_token(&transition.to);
    let writeback_action = match current.as_deref() {
        Some(value) if value == to => "already_applied",
        Some(value) if value == from => "pending_apply",
        Some(_) => "blocked_current_status",
        None => "blocked_missing_object",
    };
    SemanticLifecyclePlanEntry {
        change_intent_id: intent.id.clone(),
        object_id: transition.object_id.clone(),
        current,
        from: from.to_string(),
        to: to.to_string(),
        outcome: outcome.to_string(),
        writeback_action: writeback_action.to_string(),
    }
}

fn semantic_status_token(status: &SemanticStatus) -> &'static str {
    match status {
        SemanticStatus::Draft => "draft",
        SemanticStatus::Candidate => "candidate",
        SemanticStatus::Active => "active",
        SemanticStatus::Superseded => "superseded",
        SemanticStatus::Deprecated => "deprecated",
        SemanticStatus::Retired => "retired",
    }
}

fn render_semantic_document(frontmatter: &serde_yaml::Value, body: &str) -> Result<String> {
    let yaml =
        serde_yaml::to_string(frontmatter).context("failed to render semantic frontmatter")?;
    Ok(format!("---\n{}---\n\n{}", yaml.trim_start(), body.trim()))
}
