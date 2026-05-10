use std::collections::{BTreeMap, BTreeSet};

use crate::search::real_repo_precision::frontier::build_backend_frontier;
use crate::search::real_repo_precision::intent_frame::build_intent_frame;
use crate::search::real_repo_precision::types::{
    RealRepoKnowledgeScenario, RealRepoKnowledgeScenarioAuthorityExpectation,
    RealRepoKnowledgeScenarioAuthorityReceipt, RealRepoKnowledgeScenarioNegativeGuardReceipt,
    RealRepoKnowledgeScenarioQueryEvidenceReceipt, RealRepoKnowledgeScenarioQueryVariantReceipt,
    RealRepoKnowledgeScenarioReasoningTreeReceipt,
    RealRepoKnowledgeScenarioReasoningTreeStepReceipt, RealRepoKnowledgeScenarioReceipt,
    RealRepoMarkdownKnowledgeSemanticGateReceipt,
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt, RealRepoPrecisionQueryReceipt,
    RealRepoPrecisionRequiredPathRankReceipt,
};

pub(crate) fn evaluate_knowledge_scenario_matrix(
    scenarios: &[RealRepoKnowledgeScenario],
    query_receipts: &[RealRepoPrecisionQueryReceipt],
    semantic_gate: Option<&RealRepoMarkdownKnowledgeSemanticGateReceipt>,
) -> Vec<RealRepoKnowledgeScenarioReceipt> {
    let query_receipts_by_id = query_receipts
        .iter()
        .map(|query| (query.query_id.as_str(), query))
        .collect::<BTreeMap<_, _>>();
    let semantic_object_ids = semantic_gate
        .map(|gate| {
            gate.semantic_object_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let covered_relation_paths = semantic_gate
        .map(|gate| gate.covered_relation_paths.iter().collect())
        .unwrap_or_default();

    scenarios
        .iter()
        .map(|scenario| {
            evaluate_knowledge_scenario(
                scenario,
                &query_receipts_by_id,
                &semantic_object_ids,
                &covered_relation_paths,
            )
        })
        .collect()
}

fn evaluate_knowledge_scenario(
    scenario: &RealRepoKnowledgeScenario,
    query_receipts_by_id: &BTreeMap<&str, &RealRepoPrecisionQueryReceipt>,
    semantic_object_ids: &BTreeSet<&str>,
    covered_relation_paths: &BTreeSet<&RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
) -> RealRepoKnowledgeScenarioReceipt {
    let query_ids = scenario_query_ids(scenario);
    let query_variants = scenario_query_variants(scenario, query_receipts_by_id);
    let query_evidence = scenario_query_evidence(&query_ids, query_receipts_by_id);
    let linked_query_receipts = query_ids
        .iter()
        .filter_map(|query_id| query_receipts_by_id.get(query_id.as_str()).copied())
        .collect::<Vec<_>>();
    let coverage = evaluate_scenario_coverage(
        scenario,
        &linked_query_receipts,
        semantic_object_ids,
        covered_relation_paths,
    );
    let authority = scenario
        .authority
        .as_ref()
        .map(|expectation| evaluate_authority(expectation, &linked_query_receipts));
    let negative_guard = (!scenario.forbidden_paths.is_empty()).then(|| {
        evaluate_negative_guard(
            scenario.forbidden_paths.as_slice(),
            &coverage.observed_paths,
        )
    });

    let mut failure_reasons = scenario_failure_reasons(scenario_pass_checks(
        &query_evidence,
        &query_variants,
        &coverage,
        authority.as_ref(),
        negative_guard.as_ref(),
    ));

    let required_path_count = scenario.required_paths.len();
    let covered_required_path_count = coverage.covered_paths.len();
    let required_path_recall_bps = recall_bps(covered_required_path_count, required_path_count);
    let required_path_ranks =
        scenario_required_path_ranks(scenario.required_paths.as_slice(), &linked_query_receipts);
    let [recall_one, recall_three, recall_five, recall_ten] = path_recall_bps(&required_path_ranks);
    let mean_required_path_reciprocal_rank_bps = mean_reciprocal_rank_bps(&required_path_ranks);
    let best_required_path_rank = best_required_path_rank(&required_path_ranks);
    let reasoning_tree = build_reasoning_tree(
        scenario,
        &query_evidence,
        &required_path_ranks,
        &coverage.covered_relation_paths,
        &coverage.covered_semantic_object_ids,
    );
    if !reasoning_tree.passed {
        failure_reasons.push("reasoning_tree_failed".to_string());
    }
    let backend_frontier = build_backend_frontier(
        scenario,
        &reasoning_tree,
        authority.as_ref(),
        negative_guard.as_ref(),
    );
    let passed = failure_reasons.is_empty();
    let query_variant_count = query_variants.len();
    let passed_query_variant_count = query_variants
        .iter()
        .filter(|variant| variant.passed)
        .count();
    let failed_query_variant_count = query_variant_count.saturating_sub(passed_query_variant_count);

    RealRepoKnowledgeScenarioReceipt {
        scenario_id: scenario.id.clone(),
        scenario_kind: scenario.kind.as_str().to_string(),
        intent: scenario.intent.clone(),
        intent_frame: build_intent_frame(scenario),
        linked_query_ids: scenario.linked_query_ids.clone(),
        query_evidence,
        reasoning_tree,
        backend_frontier,
        query_variant_count,
        passed_query_variant_count,
        failed_query_variant_count,
        query_variants,
        required_path_count,
        covered_required_path_count,
        required_path_recall_bps,
        required_path_recall_at_1_bps: recall_one,
        required_path_recall_at_3_bps: recall_three,
        required_path_recall_at_5_bps: recall_five,
        required_path_recall_at_10_bps: recall_ten,
        mean_required_path_reciprocal_rank_bps,
        best_required_path_rank,
        required_path_ranks,
        required_paths: scenario.required_paths.clone(),
        covered_paths: coverage.covered_paths,
        missing_paths: coverage.missing_paths,
        required_semantic_object_ids: scenario.required_semantic_object_ids.clone(),
        covered_semantic_object_ids: coverage.covered_semantic_object_ids,
        missing_semantic_object_ids: coverage.missing_semantic_object_ids,
        required_relation_paths: scenario.required_relation_paths.clone(),
        covered_relation_paths: coverage.covered_relation_paths,
        missing_relation_paths: coverage.missing_relation_paths,
        authority,
        negative_guard,
        failure_reasons,
        passed,
    }
}

struct ScenarioCoverage<'a> {
    observed_paths: BTreeSet<&'a str>,
    covered_paths: Vec<String>,
    missing_paths: Vec<String>,
    covered_semantic_object_ids: Vec<String>,
    missing_semantic_object_ids: Vec<String>,
    covered_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    missing_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
}

fn evaluate_scenario_coverage<'a>(
    scenario: &RealRepoKnowledgeScenario,
    linked_query_receipts: &'a [&'a RealRepoPrecisionQueryReceipt],
    semantic_object_ids: &BTreeSet<&str>,
    covered_relation_paths: &BTreeSet<&RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
) -> ScenarioCoverage<'a> {
    let observed_paths = observed_paths_from_queries(linked_query_receipts);
    ScenarioCoverage {
        covered_paths: scenario
            .required_paths
            .iter()
            .filter(|path| observed_paths.contains(path.as_str()))
            .cloned()
            .collect(),
        missing_paths: scenario
            .required_paths
            .iter()
            .filter(|path| !observed_paths.contains(path.as_str()))
            .cloned()
            .collect(),
        covered_semantic_object_ids: scenario
            .required_semantic_object_ids
            .iter()
            .filter(|object_id| semantic_object_ids.contains(object_id.as_str()))
            .cloned()
            .collect(),
        missing_semantic_object_ids: scenario
            .required_semantic_object_ids
            .iter()
            .filter(|object_id| !semantic_object_ids.contains(object_id.as_str()))
            .cloned()
            .collect(),
        covered_relation_paths: scenario
            .required_relation_paths
            .iter()
            .filter(|path| covered_relation_paths.contains(*path))
            .cloned()
            .collect(),
        missing_relation_paths: scenario
            .required_relation_paths
            .iter()
            .filter(|path| !covered_relation_paths.contains(*path))
            .cloned()
            .collect(),
        observed_paths,
    }
}

fn path_recall_bps(ranks: &[RealRepoPrecisionRequiredPathRankReceipt]) -> [u32; 4] {
    [
        recall_at_bps(ranks, 1),
        recall_at_bps(ranks, 3),
        recall_at_bps(ranks, 5),
        recall_at_bps(ranks, 10),
    ]
}

fn scenario_query_variants(
    scenario: &RealRepoKnowledgeScenario,
    query_receipts_by_id: &BTreeMap<&str, &RealRepoPrecisionQueryReceipt>,
) -> Vec<RealRepoKnowledgeScenarioQueryVariantReceipt> {
    scenario
        .query_variants
        .iter()
        .map(|variant| {
            let query_evidence = query_receipts_by_id
                .get(variant.query_id.as_str())
                .map_or_else(
                    || missing_query_evidence(&variant.query_id),
                    |query| query_evidence_from_receipt(query),
                );
            RealRepoKnowledgeScenarioQueryVariantReceipt {
                query_id: variant.query_id.clone(),
                variant_kind: variant.kind.as_str().to_string(),
                passed: query_evidence.passed,
                query_evidence,
            }
        })
        .collect()
}

fn scenario_query_evidence(
    query_ids: &[String],
    query_receipts_by_id: &BTreeMap<&str, &RealRepoPrecisionQueryReceipt>,
) -> Vec<RealRepoKnowledgeScenarioQueryEvidenceReceipt> {
    query_ids
        .iter()
        .map(|query_id| {
            query_receipts_by_id.get(query_id.as_str()).map_or_else(
                || missing_query_evidence(query_id),
                |query| query_evidence_from_receipt(query),
            )
        })
        .collect()
}

fn scenario_failure_reasons(checks: [(&'static str, bool); 7]) -> Vec<String> {
    checks
        .into_iter()
        .filter_map(|(reason, passed)| (!passed).then_some(reason))
        .map(str::to_string)
        .collect()
}

fn scenario_pass_checks(
    query_evidence: &[RealRepoKnowledgeScenarioQueryEvidenceReceipt],
    query_variants: &[RealRepoKnowledgeScenarioQueryVariantReceipt],
    coverage: &ScenarioCoverage<'_>,
    authority: Option<&RealRepoKnowledgeScenarioAuthorityReceipt>,
    negative_guard: Option<&RealRepoKnowledgeScenarioNegativeGuardReceipt>,
) -> [(&'static str, bool); 7] {
    [
        (
            "query_evidence_failed",
            query_evidence.iter().all(|evidence| evidence.passed),
        ),
        (
            "query_variants_failed",
            query_variants.iter().all(|variant| variant.passed),
        ),
        ("required_paths_missing", coverage.missing_paths.is_empty()),
        (
            "semantic_objects_missing",
            coverage.missing_semantic_object_ids.is_empty(),
        ),
        (
            "relation_paths_missing",
            coverage.missing_relation_paths.is_empty(),
        ),
        (
            "authority_order_failed",
            authority.is_none_or(|authority| authority.passed),
        ),
        (
            "negative_guard_failed",
            negative_guard.is_none_or(|negative_guard| negative_guard.passed),
        ),
    ]
}

fn build_reasoning_tree(
    scenario: &RealRepoKnowledgeScenario,
    query_evidence: &[RealRepoKnowledgeScenarioQueryEvidenceReceipt],
    required_path_ranks: &[RealRepoPrecisionRequiredPathRankReceipt],
    covered_relation_paths: &[RealRepoMarkdownKnowledgeSemanticRelationPathReceipt],
    covered_semantic_object_ids: &[String],
) -> RealRepoKnowledgeScenarioReasoningTreeReceipt {
    let covered_relation_paths = covered_relation_paths.iter().collect::<BTreeSet<_>>();
    let covered_semantic_object_ids = covered_semantic_object_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut steps = Vec::new();
    let mut failure_reasons = Vec::new();

    add_anchor_steps(&mut steps, query_evidence);
    if steps.is_empty() {
        failure_reasons.push("missing_anchor".to_string());
    }

    add_relation_steps(
        &mut steps,
        &mut failure_reasons,
        &scenario.required_relation_paths,
        &covered_relation_paths,
    );
    add_page_index_seed_steps(
        &mut steps,
        &mut failure_reasons,
        &scenario.required_semantic_object_ids,
        &covered_semantic_object_ids,
    );
    add_source_evidence_steps(&mut steps, &mut failure_reasons, required_path_ranks);

    for (step_index, step) in steps.iter_mut().enumerate() {
        step.step_index = step_index;
    }
    failure_reasons.sort();
    failure_reasons.dedup();
    let step_summary = reasoning_step_summary(&steps);
    let max_disclosure_depth = steps
        .iter()
        .map(|step| step.disclosure_depth)
        .max()
        .unwrap_or_default();

    RealRepoKnowledgeScenarioReasoningTreeReceipt {
        strategy: "graph_first_progressive_disclosure_v1".to_string(),
        passed: failure_reasons.is_empty(),
        anchor_count: step_summary.anchors,
        relation_step_count: step_summary.relations,
        page_index_step_count: step_summary.page_index_seeds,
        source_step_count: step_summary.sources,
        disclosure_step_count: steps.len(),
        max_disclosure_depth,
        steps,
        failure_reasons,
    }
}

fn add_anchor_steps(
    steps: &mut Vec<RealRepoKnowledgeScenarioReasoningTreeStepReceipt>,
    query_evidence: &[RealRepoKnowledgeScenarioQueryEvidenceReceipt],
) {
    for evidence in query_evidence.iter().filter(|evidence| evidence.passed) {
        steps.push(reasoning_step(ReasoningStepInput {
            step_kind: "anchor_query",
            evidence_id: format!("anchor:{}", evidence.query_id),
            query_id: Some(evidence.query_id.clone()),
            path: evidence.observed_top_path.clone(),
            relation: None,
            semantic_object_id: None,
            zero_based_rank: evidence.best_required_path_rank,
            disclosure_depth: 0,
            passed: evidence.observed_top_path.is_some(),
            failure_reason: evidence
                .observed_top_path
                .is_none()
                .then(|| "anchor query did not return an observed top path".to_string()),
        }));
    }
}

fn add_relation_steps(
    steps: &mut Vec<RealRepoKnowledgeScenarioReasoningTreeStepReceipt>,
    failure_reasons: &mut Vec<String>,
    required_relation_paths: &[RealRepoMarkdownKnowledgeSemanticRelationPathReceipt],
    covered_relation_paths: &BTreeSet<&RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
) {
    for relation in required_relation_paths {
        let passed = covered_relation_paths.contains(relation);
        steps.push(reasoning_step(ReasoningStepInput {
            step_kind: "semantic_relation",
            evidence_id: format!(
                "relation:{}:{}:{}",
                relation.source, relation.kind, relation.target
            ),
            query_id: None,
            path: None,
            relation: Some(relation.clone()),
            semantic_object_id: None,
            zero_based_rank: None,
            disclosure_depth: 1,
            passed,
            failure_reason: (!passed)
                .then(|| "required semantic relation was not covered".to_string()),
        }));
        if !passed {
            failure_reasons.push("missing_relation_step".to_string());
        }
    }
}

fn add_page_index_seed_steps(
    steps: &mut Vec<RealRepoKnowledgeScenarioReasoningTreeStepReceipt>,
    failure_reasons: &mut Vec<String>,
    required_semantic_object_ids: &[String],
    covered_semantic_object_ids: &BTreeSet<&str>,
) {
    for object_id in required_semantic_object_ids {
        let passed = covered_semantic_object_ids.contains(object_id.as_str());
        steps.push(reasoning_step(ReasoningStepInput {
            step_kind: "page_index_seed",
            evidence_id: format!("page-index-seed:{object_id}"),
            query_id: None,
            path: None,
            relation: None,
            semantic_object_id: Some(object_id.clone()),
            zero_based_rank: None,
            disclosure_depth: 2,
            passed,
            failure_reason: (!passed).then(|| {
                "required semantic object was not available as PageIndex seed evidence".to_string()
            }),
        }));
        if !passed {
            failure_reasons.push("missing_page_index_seed".to_string());
        }
    }
}

fn add_source_evidence_steps(
    steps: &mut Vec<RealRepoKnowledgeScenarioReasoningTreeStepReceipt>,
    failure_reasons: &mut Vec<String>,
    required_path_ranks: &[RealRepoPrecisionRequiredPathRankReceipt],
) {
    for rank in required_path_ranks {
        let passed = rank.zero_based_rank.is_some();
        steps.push(reasoning_step(ReasoningStepInput {
            step_kind: "source_evidence",
            evidence_id: format!("source:{}", rank.path),
            query_id: None,
            path: Some(rank.path.clone()),
            relation: None,
            semantic_object_id: None,
            zero_based_rank: rank.zero_based_rank,
            disclosure_depth: 2,
            passed,
            failure_reason: (!passed).then(|| "required source path was not observed".to_string()),
        }));
        if !passed {
            failure_reasons.push("missing_source_evidence".to_string());
        }
    }
}

struct ReasoningStepSummary {
    anchors: usize,
    relations: usize,
    page_index_seeds: usize,
    sources: usize,
}

fn reasoning_step_summary(
    steps: &[RealRepoKnowledgeScenarioReasoningTreeStepReceipt],
) -> ReasoningStepSummary {
    ReasoningStepSummary {
        anchors: passed_step_count(steps, "anchor_query"),
        relations: passed_step_count(steps, "semantic_relation"),
        page_index_seeds: passed_step_count(steps, "page_index_seed"),
        sources: passed_step_count(steps, "source_evidence"),
    }
}

fn passed_step_count(
    steps: &[RealRepoKnowledgeScenarioReasoningTreeStepReceipt],
    step_kind: &str,
) -> usize {
    steps
        .iter()
        .filter(|step| step.step_kind == step_kind && step.passed)
        .count()
}

struct ReasoningStepInput {
    step_kind: &'static str,
    evidence_id: String,
    query_id: Option<String>,
    path: Option<String>,
    relation: Option<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    semantic_object_id: Option<String>,
    zero_based_rank: Option<usize>,
    disclosure_depth: usize,
    passed: bool,
    failure_reason: Option<String>,
}

fn reasoning_step(input: ReasoningStepInput) -> RealRepoKnowledgeScenarioReasoningTreeStepReceipt {
    RealRepoKnowledgeScenarioReasoningTreeStepReceipt {
        step_index: 0,
        step_kind: input.step_kind.to_string(),
        evidence_id: input.evidence_id,
        query_id: input.query_id,
        path: input.path,
        relation: input.relation,
        semantic_object_id: input.semantic_object_id,
        zero_based_rank: input.zero_based_rank,
        disclosure_depth: input.disclosure_depth,
        passed: input.passed,
        failure_reason: input.failure_reason,
    }
}

fn scenario_query_ids(scenario: &RealRepoKnowledgeScenario) -> Vec<String> {
    let mut query_ids = scenario
        .linked_query_ids
        .iter()
        .cloned()
        .chain(
            scenario
                .query_variants
                .iter()
                .map(|variant| variant.query_id.clone()),
        )
        .collect::<Vec<_>>();
    query_ids.sort();
    query_ids.dedup();
    query_ids
}

fn query_evidence_from_receipt(
    query: &RealRepoPrecisionQueryReceipt,
) -> RealRepoKnowledgeScenarioQueryEvidenceReceipt {
    RealRepoKnowledgeScenarioQueryEvidenceReceipt {
        query_id: query.query_id.clone(),
        query_kind: query.query_kind.clone(),
        query_ms: query.query_ms,
        passed: query.passed,
        required_top_path: query.required_top_path.clone(),
        observed_top_path: query.observed_top_path.clone(),
        missing_paths: query.missing_paths.clone(),
        required_path_ranks: query.required_path_ranks.clone(),
        required_path_recall_at_1_bps: query.required_path_recall_at_1_bps,
        required_path_recall_at_3_bps: query.required_path_recall_at_3_bps,
        required_path_recall_at_5_bps: query.required_path_recall_at_5_bps,
        required_path_recall_at_10_bps: query.required_path_recall_at_10_bps,
        mean_required_path_reciprocal_rank_bps: query.mean_required_path_reciprocal_rank_bps,
        best_required_path_rank: query.best_required_path_rank,
        observed_path_count: query.observed_paths.len(),
        failure_reason: (!query.passed).then(|| "query receipt failed".to_string()),
    }
}

fn missing_query_evidence(query_id: &str) -> RealRepoKnowledgeScenarioQueryEvidenceReceipt {
    RealRepoKnowledgeScenarioQueryEvidenceReceipt {
        query_id: query_id.to_string(),
        query_kind: "missing".to_string(),
        query_ms: 0,
        passed: false,
        required_top_path: None,
        observed_top_path: None,
        missing_paths: Vec::new(),
        required_path_ranks: Vec::new(),
        required_path_recall_at_1_bps: 0,
        required_path_recall_at_3_bps: 0,
        required_path_recall_at_5_bps: 0,
        required_path_recall_at_10_bps: 0,
        mean_required_path_reciprocal_rank_bps: 0,
        best_required_path_rank: None,
        observed_path_count: 0,
        failure_reason: Some("query receipt missing".to_string()),
    }
}

fn observed_paths_from_queries<'a>(
    query_receipts: &'a [&'a RealRepoPrecisionQueryReceipt],
) -> BTreeSet<&'a str> {
    query_receipts
        .iter()
        .flat_map(|query| query.observed_paths.iter().map(String::as_str))
        .collect()
}

fn evaluate_authority(
    expectation: &RealRepoKnowledgeScenarioAuthorityExpectation,
    query_receipts: &[&RealRepoPrecisionQueryReceipt],
) -> RealRepoKnowledgeScenarioAuthorityReceipt {
    let preferred_rank = first_observed_rank(query_receipts, &expectation.preferred_path);
    let earliest_competing_rank = expectation
        .competing_paths
        .iter()
        .filter_map(|path| first_observed_rank(query_receipts, path))
        .min();
    let passed = preferred_rank.is_some_and(|preferred| {
        earliest_competing_rank.is_none_or(|competing| preferred < competing)
    });
    let failure_reason = (!passed).then(|| {
        if preferred_rank.is_none() {
            "preferred path was not observed".to_string()
        } else {
            "a competing path ranked before or equal to the preferred path".to_string()
        }
    });

    RealRepoKnowledgeScenarioAuthorityReceipt {
        preferred_path: expectation.preferred_path.clone(),
        competing_paths: expectation.competing_paths.clone(),
        preferred_rank,
        earliest_competing_rank,
        passed,
        failure_reason,
    }
}

fn evaluate_negative_guard(
    forbidden_paths: &[String],
    observed_paths: &BTreeSet<&str>,
) -> RealRepoKnowledgeScenarioNegativeGuardReceipt {
    let matched_forbidden_paths = forbidden_paths
        .iter()
        .filter(|path| observed_paths.contains(path.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    RealRepoKnowledgeScenarioNegativeGuardReceipt {
        forbidden_paths: forbidden_paths.to_vec(),
        passed: matched_forbidden_paths.is_empty(),
        matched_forbidden_paths,
    }
}

fn first_observed_rank(
    query_receipts: &[&RealRepoPrecisionQueryReceipt],
    path: &str,
) -> Option<usize> {
    query_receipts
        .iter()
        .flat_map(|query| query.observed_paths.iter().enumerate())
        .filter_map(|(rank, observed_path)| (observed_path == path).then_some(rank))
        .min()
}

fn scenario_required_path_ranks(
    required_paths: &[String],
    query_receipts: &[&RealRepoPrecisionQueryReceipt],
) -> Vec<RealRepoPrecisionRequiredPathRankReceipt> {
    required_paths
        .iter()
        .map(|path| RealRepoPrecisionRequiredPathRankReceipt {
            path: path.clone(),
            zero_based_rank: query_receipts
                .iter()
                .flat_map(|query| query.observed_paths.iter().enumerate())
                .filter_map(|(rank, observed)| (observed == path).then_some(rank))
                .min(),
        })
        .collect()
}

fn recall_at_bps(ranks: &[RealRepoPrecisionRequiredPathRankReceipt], k: usize) -> u32 {
    if ranks.is_empty() {
        return 10_000;
    }
    let covered = ranks
        .iter()
        .filter(|rank| rank.zero_based_rank.is_some_and(|value| value < k))
        .count();
    usize_to_u32_saturating((covered * 10_000) / ranks.len())
}

fn mean_reciprocal_rank_bps(ranks: &[RealRepoPrecisionRequiredPathRankReceipt]) -> u32 {
    if ranks.is_empty() {
        return 10_000;
    }
    let total = ranks
        .iter()
        .map(|rank| {
            rank.zero_based_rank.map_or(0, |value| {
                10_000 / usize_to_u32_saturating(value.saturating_add(1))
            })
        })
        .sum::<u32>();
    total / usize_to_u32_saturating(ranks.len())
}

fn best_required_path_rank(ranks: &[RealRepoPrecisionRequiredPathRankReceipt]) -> Option<usize> {
    ranks.iter().filter_map(|rank| rank.zero_based_rank).min()
}

fn recall_bps(covered: usize, required: usize) -> u32 {
    if required == 0 {
        return 10_000;
    }
    usize_to_u32_saturating((covered * 10_000) / required)
}

fn usize_to_u32_saturating(value: usize) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
