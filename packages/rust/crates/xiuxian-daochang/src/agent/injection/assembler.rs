use std::collections::HashSet;

use xiuxian_qianhuan::{
    InjectionMode, InjectionOrderStrategy, InjectionPolicy, InjectionSessionId, InjectionSnapshot,
    InjectionSnapshotId, InjectionSnapshotInput, InjectionTurnId, PromptContextBlock,
    PromptContextBlockId, PromptContextBlockInput, PromptContextCategory, PromptContextSource,
    PromptSessionScope, RoleMixProfile, RoleMixRole,
};

use super::builder::InjectionBlock;

pub(super) fn assemble_snapshot(
    session_id: &str,
    turn_id: u64,
    policy: InjectionPolicy,
    blocks: Vec<InjectionBlock>,
) -> InjectionSnapshot {
    let blocks: Vec<PromptContextBlock> = blocks
        .into_iter()
        .map(|block| {
            PromptContextBlock::new(PromptContextBlockInput {
                block_id: PromptContextBlockId::new(block.block_id),
                source: PromptContextSource::RuntimeHint,
                category: PromptContextCategory::RuntimeHint,
                priority: 0,
                session_scope: PromptSessionScope::new(session_id),
                payload: block.content,
                anchor: false,
            })
        })
        .collect();
    let mut dropped_block_ids = Vec::new();

    let mut selected = blocks
        .into_iter()
        .filter_map(|mut block| {
            if !policy.enabled_categories.contains(&block.category) {
                dropped_block_ids.push(block.block_id.to_string());
                return None;
            }
            block.anchor = block.anchor || policy.anchor_categories.contains(&block.category);
            Some(block)
        })
        .collect::<Vec<_>>();

    sort_blocks(&mut selected, &policy);
    let role_mix = Some(select_role_mix(&policy, &selected));

    let mut retained = Vec::new();
    for block in selected {
        if retained.len() < policy.max_blocks {
            retained.push(block);
            continue;
        }

        if block.anchor
            && let Some(replace_index) = retained.iter().rposition(|existing| !existing.anchor)
        {
            let evicted = std::mem::replace(&mut retained[replace_index], block);
            dropped_block_ids.push(evicted.block_id.to_string());
            continue;
        }

        dropped_block_ids.push(block.block_id.to_string());
    }

    let (final_blocks, mut budget_dropped, truncated_block_ids) =
        apply_char_budget(prioritize_anchors(retained), policy.max_chars);
    dropped_block_ids.append(&mut budget_dropped);

    let mut snapshot = InjectionSnapshot::from_blocks(InjectionSnapshotInput {
        snapshot_id: InjectionSnapshotId::new(format!("injection:{session_id}:{turn_id}")),
        session_id: InjectionSessionId::new(session_id),
        turn_id: InjectionTurnId::new(turn_id),
        policy,
        role_mix,
        blocks: final_blocks,
    });
    snapshot.dropped_block_ids = dedup_preserve_order(dropped_block_ids);
    snapshot.truncated_block_ids = dedup_preserve_order(truncated_block_ids);
    snapshot
}

fn sort_blocks(blocks: &mut [PromptContextBlock], policy: &InjectionPolicy) {
    match policy.ordering {
        InjectionOrderStrategy::PriorityDesc => {
            blocks.sort_by(|left, right| {
                right
                    .priority
                    .cmp(&left.priority)
                    .then_with(|| left.block_id.cmp(&right.block_id))
            });
        }
        InjectionOrderStrategy::CategoryThenPriority => {
            blocks.sort_by(|left, right| {
                category_rank(&policy.enabled_categories, left.category)
                    .cmp(&category_rank(&policy.enabled_categories, right.category))
                    .then_with(|| right.priority.cmp(&left.priority))
                    .then_with(|| left.block_id.cmp(&right.block_id))
            });
        }
    }
}

fn category_rank(enabled: &[PromptContextCategory], category: PromptContextCategory) -> usize {
    enabled
        .iter()
        .position(|value| *value == category)
        .unwrap_or(usize::MAX)
}

fn apply_char_budget(
    blocks: Vec<PromptContextBlock>,
    max_chars: usize,
) -> (Vec<PromptContextBlock>, Vec<String>, Vec<String>) {
    if max_chars == 0 {
        let dropped = blocks
            .into_iter()
            .map(|block| block.block_id.to_string())
            .collect();
        return (Vec::new(), dropped, Vec::new());
    }

    let mut kept = Vec::new();
    let mut dropped_block_ids = Vec::new();
    let mut truncated_block_ids = Vec::new();
    let mut used_chars = 0usize;

    for mut block in blocks {
        if used_chars >= max_chars {
            dropped_block_ids.push(block.block_id.to_string());
            continue;
        }

        let remaining = max_chars.saturating_sub(used_chars);
        if block.payload_chars <= remaining {
            used_chars = used_chars.saturating_add(block.payload_chars);
            kept.push(block);
            continue;
        }

        if remaining == 0 {
            dropped_block_ids.push(block.block_id.to_string());
            continue;
        }

        block.payload = truncate_chars(&block.payload, remaining);
        block.payload_chars = block.payload.chars().count();
        used_chars = used_chars.saturating_add(block.payload_chars);
        truncated_block_ids.push(block.block_id.to_string());
        kept.push(block);
    }

    (kept, dropped_block_ids, truncated_block_ids)
}

fn prioritize_anchors(blocks: Vec<PromptContextBlock>) -> Vec<PromptContextBlock> {
    let (anchors, others): (Vec<_>, Vec<_>) = blocks.into_iter().partition(|block| block.anchor);
    anchors.into_iter().chain(others).collect()
}

fn select_role_mix(policy: &InjectionPolicy, blocks: &[PromptContextBlock]) -> RoleMixProfile {
    let mut roles = collect_role_candidates(blocks);
    if roles.is_empty() {
        roles.push(default_role_mix_role());
    }
    build_role_mix_profile(policy.mode, roles)
}

fn collect_role_candidates(blocks: &[PromptContextBlock]) -> Vec<RoleMixRole> {
    let mut roles = Vec::new();
    let mut seen = HashSet::new();

    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[PromptContextCategory::Safety, PromptContextCategory::Policy],
        "governance_guardian",
        0.36,
        "safety/policy anchors present",
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[
            PromptContextCategory::MemoryRecall,
            PromptContextCategory::WindowSummary,
        ],
        "memory_strategist",
        0.31,
        "memory recall or window summary context present",
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[PromptContextCategory::SessionXml],
        "session_context_curator",
        0.27,
        "session XML context present",
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[PromptContextCategory::Knowledge],
        "knowledge_synthesizer",
        0.33,
        "knowledge retrieval context present",
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[
            PromptContextCategory::Reflection,
            PromptContextCategory::RuntimeHint,
        ],
        "reflection_optimizer",
        0.29,
        "reflection/runtime hint context present",
    );

    roles
}

fn build_role_mix_profile(mode: InjectionMode, roles: Vec<RoleMixRole>) -> RoleMixProfile {
    match mode {
        InjectionMode::Single => {
            let primary = roles.first().cloned().unwrap_or_else(default_role_mix_role);
            RoleMixProfile {
                profile_id: "role_mix.single.v1".to_string(),
                roles: vec![primary.clone()],
                rationale: format!(
                    "policy.mode=single selected deterministic primary role `{}`",
                    primary.role
                ),
            }
        }
        InjectionMode::Classified => RoleMixProfile {
            profile_id: "role_mix.classified.v1".to_string(),
            rationale: format!(
                "policy.mode=classified selected {} role domains from retained blocks",
                roles.len()
            ),
            roles,
        },
        InjectionMode::Hybrid => RoleMixProfile {
            profile_id: "role_mix.hybrid.v1".to_string(),
            rationale: format!(
                "policy.mode=hybrid selected {} role domains for mixed-context synthesis",
                roles.len()
            ),
            roles,
        },
    }
}

fn maybe_push_role_for_categories(
    blocks: &[PromptContextBlock],
    roles: &mut Vec<RoleMixRole>,
    seen: &mut HashSet<&'static str>,
    categories: &[PromptContextCategory],
    role: &'static str,
    weight: f32,
    reason: &'static str,
) {
    if has_any_category(blocks, categories) {
        push_role(roles, seen, role, weight, reason);
    }
}

fn has_any_category(blocks: &[PromptContextBlock], categories: &[PromptContextCategory]) -> bool {
    blocks
        .iter()
        .any(|block| categories.contains(&block.category))
}

fn default_role_mix_role() -> RoleMixRole {
    RoleMixRole {
        role: "session_context_curator".to_string(),
        weight: 1.0,
    }
}

fn push_role(
    roles: &mut Vec<RoleMixRole>,
    seen: &mut HashSet<&'static str>,
    role: &'static str,
    weight: f32,
    _reason: &'static str,
) {
    if seen.insert(role) {
        roles.push(RoleMixRole {
            role: role.to_string(),
            weight,
        });
    }
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut out = input
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    out.push_str("...");
    out
}

fn dedup_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}
