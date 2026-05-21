use std::collections::BTreeSet;

use crate::search::real_repo_precision::types::{
    RealRepoKnowledgeScenario, RealRepoKnowledgeScenarioIntentFrameReceipt,
};

pub(crate) fn build_intent_frame(
    scenario: &RealRepoKnowledgeScenario,
) -> RealRepoKnowledgeScenarioIntentFrameReceipt {
    let mut required_evidence_kinds = BTreeSet::new();
    if !scenario.required_paths.is_empty() {
        required_evidence_kinds.insert("source_path".to_string());
    }
    if !scenario.required_semantic_object_ids.is_empty() {
        required_evidence_kinds.insert("semantic_object".to_string());
        required_evidence_kinds.insert("page_index_seed".to_string());
    }
    if !scenario.required_relation_paths.is_empty() {
        required_evidence_kinds.insert("relation_path".to_string());
    }
    if scenario.authority.is_some() {
        required_evidence_kinds.insert("authority_order".to_string());
    }
    if !scenario.forbidden_paths.is_empty() {
        required_evidence_kinds.insert("negative_guard".to_string());
    }

    RealRepoKnowledgeScenarioIntentFrameReceipt {
        task_kind: scenario.kind.as_str().to_string(),
        anchor_terms: intent_anchor_terms(&scenario.intent),
        required_evidence_kinds: required_evidence_kinds.into_iter().collect(),
        relation_hypotheses: scenario.required_relation_paths.clone(),
        authority_policy: intent_authority_policy(scenario),
        max_disclosure_depth: intent_max_disclosure_depth(scenario),
        verifier_required: true,
    }
}

fn intent_anchor_terms(intent: &str) -> Vec<String> {
    let stopwords = [
        "about",
        "after",
        "against",
        "answer",
        "before",
        "broad",
        "canonical",
        "changing",
        "document",
        "evidence",
        "explicit",
        "find",
        "from",
        "gather",
        "into",
        "needs",
        "rather",
        "retrieving",
        "than",
        "that",
        "the",
        "when",
        "where",
        "with",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut terms = intent
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .map(str::trim)
        .filter(|term| term.len() >= 4)
        .map(str::to_ascii_lowercase)
        .filter(|term| !stopwords.contains(term.as_str()))
        .collect::<BTreeSet<_>>();
    while terms.len() > 12 {
        let Some(last) = terms.iter().next_back().cloned() else {
            break;
        };
        terms.remove(&last);
    }
    terms.into_iter().collect()
}

fn intent_authority_policy(scenario: &RealRepoKnowledgeScenario) -> Vec<String> {
    if let Some(authority) = scenario.authority.as_ref() {
        let mut policy = vec![format!("prefer:{}", authority.preferred_path)];
        policy.extend(
            authority
                .competing_paths
                .iter()
                .map(|path| format!("deprioritize:{path}")),
        );
        return policy;
    }
    vec![
        "semantic_ssot_before_package_docs".to_string(),
        "rfc_before_feature_notes".to_string(),
        "source_paths_are_required_evidence".to_string(),
    ]
}

fn intent_max_disclosure_depth(scenario: &RealRepoKnowledgeScenario) -> usize {
    if !scenario.required_relation_paths.is_empty()
        || !scenario.required_semantic_object_ids.is_empty()
    {
        return 2;
    }
    1
}
