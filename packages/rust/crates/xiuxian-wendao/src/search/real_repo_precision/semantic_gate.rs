use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use xiuxian_wendao_parsers::semantic_ssot::{
    SemanticObject, SemanticRelationEdge, SemanticRelationKind, SemanticScopeBundle,
    SemanticScopeRequest, load_semantic_repository, semantic_scope_bundle,
};

use crate::link_graph::{
    WendaoGraphPageIndexReasoningRequestBundle,
    build_semantic_scope_page_index_reasoning_request_bundle,
};
use crate::search::real_repo_precision::types::{
    RealRepoGoldQuery, RealRepoGoldQueryKind, RealRepoMarkdownKnowledgeSemanticGateReceipt,
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt,
    RealRepoMarkdownKnowledgeSemanticScenarioQueryEvidenceReceipt,
    RealRepoMarkdownKnowledgeSemanticScenarioReceipt, RealRepoPrecisionQueryReceipt,
};

const GATE_SCHEMA: &str = "xiuxian_wendao.real_repo_markdown_knowledge_semantic_gate.v1";

struct LinkedSemanticGoldQuery {
    query_id: &'static str,
    object_id: &'static str,
}

const LINKED_SEMANTIC_GOLD_QUERIES: &[LinkedSemanticGoldQuery] = &[
    LinkedSemanticGoldQuery {
        query_id: "repo-native-semantic-ssot-rfc",
        object_id: "component.wendao.query-substrate",
    },
    LinkedSemanticGoldQuery {
        query_id: "semantic-object-wendao-query-substrate",
        object_id: "component.wendao.query-substrate",
    },
    LinkedSemanticGoldQuery {
        query_id: "semantic-decision-repo-native-authority",
        object_id: "decision.semantic-ssot.repo-native-first",
    },
    LinkedSemanticGoldQuery {
        query_id: "semantic-decision-projections-read-models",
        object_id: "decision.semantic-ssot.projections-are-read-models",
    },
    LinkedSemanticGoldQuery {
        query_id: "semantic-invariant-llm-output-not-authority",
        object_id: "invariant.llm-output-is-not-authority",
    },
    LinkedSemanticGoldQuery {
        query_id: "semantic-relation-repo-native-governs-query-substrate",
        object_id: "decision.semantic-ssot.repo-native-first",
    },
    LinkedSemanticGoldQuery {
        query_id: "semantic-relation-projections-govern-llm-boundary",
        object_id: "decision.semantic-ssot.projections-are-read-models",
    },
    LinkedSemanticGoldQuery {
        query_id: "semantic-relation-llm-constrains-projections",
        object_id: "invariant.llm-output-is-not-authority",
    },
];

#[derive(Clone, Copy)]
struct RequiredSemanticRelationPath {
    source: &'static str,
    kind: &'static str,
    target: &'static str,
}

const REPO_NATIVE_GOVERNS_QUERY_SUBSTRATE: RequiredSemanticRelationPath =
    RequiredSemanticRelationPath {
        source: "decision.semantic-ssot.repo-native-first",
        kind: "governs",
        target: "component.wendao.query-substrate",
    };
const PROJECTIONS_GOVERN_LLM_BOUNDARY: RequiredSemanticRelationPath =
    RequiredSemanticRelationPath {
        source: "decision.semantic-ssot.projections-are-read-models",
        kind: "governs",
        target: "invariant.llm-output-is-not-authority",
    };
const LLM_CONSTRAINS_PROJECTIONS: RequiredSemanticRelationPath = RequiredSemanticRelationPath {
    source: "invariant.llm-output-is-not-authority",
    kind: "constrains",
    target: "decision.semantic-ssot.projections-are-read-models",
};
const TASK_VALIDATES_LLM_BOUNDARY: RequiredSemanticRelationPath = RequiredSemanticRelationPath {
    source: "task.semantic-ssot.object-schema-pilot",
    kind: "validates",
    target: "invariant.llm-output-is-not-authority",
};

const REQUIRED_SEMANTIC_RELATION_PATHS: &[RequiredSemanticRelationPath] = &[
    REPO_NATIVE_GOVERNS_QUERY_SUBSTRATE,
    PROJECTIONS_GOVERN_LLM_BOUNDARY,
    LLM_CONSTRAINS_PROJECTIONS,
    TASK_VALIDATES_LLM_BOUNDARY,
];

struct RequiredSemanticScenario {
    scenario_id: &'static str,
    intent: &'static str,
    linked_query_ids: &'static [&'static str],
    required_object_ids: &'static [&'static str],
    required_relation_paths: &'static [RequiredSemanticRelationPath],
}

const SCENARIO_WENDAO_QUERY_SUBSTRATE_RELATIONS: &[RequiredSemanticRelationPath] =
    &[REPO_NATIVE_GOVERNS_QUERY_SUBSTRATE];
const SCENARIO_PROJECTION_BOUNDARY_RELATIONS: &[RequiredSemanticRelationPath] =
    &[PROJECTIONS_GOVERN_LLM_BOUNDARY, LLM_CONSTRAINS_PROJECTIONS];
const SCENARIO_LLM_AUTHORITY_RELATIONS: &[RequiredSemanticRelationPath] =
    &[LLM_CONSTRAINS_PROJECTIONS, TASK_VALIDATES_LLM_BOUNDARY];

const REQUIRED_SEMANTIC_SCENARIOS: &[RequiredSemanticScenario] = &[
    RequiredSemanticScenario {
        scenario_id: "wendao-query-substrate-authority",
        intent: "Find the authoritative Wendao query substrate and its repo-native governance decision.",
        linked_query_ids: &[
            "repo-native-semantic-ssot-rfc",
            "semantic-object-wendao-query-substrate",
            "semantic-relation-repo-native-governs-query-substrate",
        ],
        required_object_ids: &[
            "component.wendao.query-substrate",
            "decision.semantic-ssot.repo-native-first",
        ],
        required_relation_paths: SCENARIO_WENDAO_QUERY_SUBSTRATE_RELATIONS,
    },
    RequiredSemanticScenario {
        scenario_id: "projection-read-model-authority-boundary",
        intent: "Explain why projections and LLM summaries are read models, not semantic authority.",
        linked_query_ids: &[
            "semantic-decision-projections-read-models",
            "semantic-invariant-llm-output-not-authority",
            "semantic-relation-projections-govern-llm-boundary",
            "semantic-relation-llm-constrains-projections",
        ],
        required_object_ids: &[
            "decision.semantic-ssot.projections-are-read-models",
            "invariant.llm-output-is-not-authority",
        ],
        required_relation_paths: SCENARIO_PROJECTION_BOUNDARY_RELATIONS,
    },
    RequiredSemanticScenario {
        scenario_id: "llm-output-authority-validation",
        intent: "Validate the LLM-output authority boundary through SSOT invariants and schema-task evidence.",
        linked_query_ids: &[
            "semantic-invariant-llm-output-not-authority",
            "semantic-relation-llm-constrains-projections",
        ],
        required_object_ids: &[
            "decision.semantic-ssot.projections-are-read-models",
            "invariant.llm-output-is-not-authority",
            "task.semantic-ssot.object-schema-pilot",
        ],
        required_relation_paths: SCENARIO_LLM_AUTHORITY_RELATIONS,
    },
];

pub(crate) struct RealRepoMarkdownKnowledgeSemanticGateEvaluation {
    pub(crate) receipt: RealRepoMarkdownKnowledgeSemanticGateReceipt,
    pub(crate) page_index: WendaoGraphPageIndexReasoningRequestBundle,
}

pub(crate) fn evaluate_markdown_knowledge_semantic_gate(
    semantic_root: &Path,
    gold_queries: &[RealRepoGoldQuery],
) -> Result<Option<RealRepoMarkdownKnowledgeSemanticGateEvaluation>, String> {
    let linked_queries = linked_semantic_gold_queries(gold_queries);
    if linked_queries.is_empty() {
        return Ok(None);
    }
    if !semantic_root.is_dir() {
        return Err(format!(
            "semantic root `{}` is missing for linked Markdown knowledge queries",
            semantic_root.display()
        ));
    }

    let repository = load_valid_semantic_repository(semantic_root)?;
    let scope = linked_query_semantic_scope(&repository, &linked_queries)?;
    let required_markdown_paths = required_markdown_paths(&linked_queries);
    let covered_markdown_paths =
        covered_markdown_paths_for_linked_queries(&scope, &linked_queries)?;
    validate_markdown_path_coverage(&required_markdown_paths, &covered_markdown_paths)?;
    let required_relation_paths = required_semantic_relation_path_receipts();
    let covered_relation_paths = covered_semantic_relation_paths(&scope.relations);
    validate_relation_path_coverage(&required_relation_paths, &covered_relation_paths)?;

    let page_index = build_semantic_scope_page_index_reasoning_request_bundle(&scope)
        .map_err(|error| format!("build semantic PageIndex request bundle: {error}"))?;
    let linked_query_ids = linked_query_ids(&linked_queries);
    let semantic_object_ids = semantic_object_ids(&scope);
    let knowledge_scenarios = semantic_knowledge_scenarios(
        &linked_query_ids,
        &semantic_object_ids,
        &covered_relation_paths,
    );
    let receipt = RealRepoMarkdownKnowledgeSemanticGateReceipt {
        schema: GATE_SCHEMA.to_string(),
        semantic_root: semantic_root.display().to_string(),
        linked_query_ids,
        required_markdown_paths,
        covered_markdown_paths,
        required_relation_paths,
        covered_relation_paths,
        knowledge_scenarios,
        semantic_object_ids,
        semantic_scope_object_count: scope.objects.len(),
        semantic_scope_relation_count: scope.relations.len(),
        page_index_node_count: page_index.nodes.num_rows(),
        page_index_edge_count: page_index.edges.num_rows(),
        page_index_seed_count: page_index.seeds.num_rows(),
        required_validation_count: scope.required_validations.len(),
    };

    Ok(Some(RealRepoMarkdownKnowledgeSemanticGateEvaluation {
        receipt,
        page_index,
    }))
}

fn load_valid_semantic_repository(
    semantic_root: &Path,
) -> Result<xiuxian_wendao_parsers::semantic_ssot::SemanticRepository, String> {
    let repository = load_semantic_repository(semantic_root);
    if repository.report.is_success() {
        return Ok(repository);
    }
    let issues = repository
        .report
        .issues
        .iter()
        .map(|issue| {
            issue.path.as_ref().map_or_else(
                || issue.message.clone(),
                |path| format!("{}: {}", path.display(), issue.message),
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    Err(format!(
        "semantic repository `{}` failed validation: {issues}",
        semantic_root.display()
    ))
}

fn linked_query_semantic_scope(
    repository: &xiuxian_wendao_parsers::semantic_ssot::SemanticRepository,
    linked_queries: &[(&RealRepoGoldQuery, &'static LinkedSemanticGoldQuery)],
) -> Result<SemanticScopeBundle, String> {
    let object_ids = linked_queries
        .iter()
        .map(|(_, link)| link.object_id.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let scope = semantic_scope_bundle(
        repository,
        &SemanticScopeRequest {
            task_id: None,
            object_ids,
        },
    );
    if scope.unresolved_ids.is_empty() {
        return Ok(scope);
    }
    Err(format!(
        "semantic scope unresolved ids: {}",
        scope.unresolved_ids.join(",")
    ))
}

fn required_markdown_paths(
    linked_queries: &[(&RealRepoGoldQuery, &'static LinkedSemanticGoldQuery)],
) -> Vec<String> {
    linked_queries
        .iter()
        .flat_map(|(query, _)| query.must_hit_paths.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn validate_markdown_path_coverage(
    required_markdown_paths: &[String],
    covered_markdown_paths: &[String],
) -> Result<(), String> {
    let missing_markdown_paths = required_markdown_paths
        .iter()
        .filter(|path| !covered_markdown_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    if missing_markdown_paths.is_empty() {
        return Ok(());
    }
    Err(format!(
        "semantic gate missing Markdown path coverage: {}",
        missing_markdown_paths.join(",")
    ))
}

fn validate_relation_path_coverage(
    required_relation_paths: &[RealRepoMarkdownKnowledgeSemanticRelationPathReceipt],
    covered_relation_paths: &[RealRepoMarkdownKnowledgeSemanticRelationPathReceipt],
) -> Result<(), String> {
    let missing_relation_paths = required_relation_paths
        .iter()
        .filter(|path| !covered_relation_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    if missing_relation_paths.is_empty() {
        return Ok(());
    }
    let missing = missing_relation_paths
        .iter()
        .map(relation_path_label)
        .collect::<Vec<_>>()
        .join(",");
    Err(format!("semantic gate missing relation paths: {missing}"))
}

fn linked_query_ids(
    linked_queries: &[(&RealRepoGoldQuery, &'static LinkedSemanticGoldQuery)],
) -> Vec<String> {
    linked_queries
        .iter()
        .map(|(query, _)| query.id.clone())
        .collect()
}

fn semantic_object_ids(scope: &SemanticScopeBundle) -> Vec<String> {
    scope
        .objects
        .iter()
        .map(|object| object.id.clone())
        .collect()
}

pub(crate) fn attach_markdown_knowledge_semantic_query_evidence(
    receipt: &mut RealRepoMarkdownKnowledgeSemanticGateReceipt,
    query_receipts: &[RealRepoPrecisionQueryReceipt],
) {
    let query_receipts_by_id = query_receipts
        .iter()
        .map(|query| (query.query_id.as_str(), query))
        .collect::<BTreeMap<_, _>>();

    for scenario in &mut receipt.knowledge_scenarios {
        scenario.query_evidence = scenario
            .linked_query_ids
            .iter()
            .map(|query_id| {
                query_receipts_by_id.get(query_id.as_str()).map_or_else(
                    || missing_query_evidence(query_id),
                    |query| query_evidence_from_receipt(query),
                )
            })
            .collect();
        let query_evidence_passed = scenario
            .query_evidence
            .iter()
            .all(|evidence| evidence.passed);
        scenario.passed = scenario.passed && query_evidence_passed;
    }
}

fn query_evidence_from_receipt(
    query: &RealRepoPrecisionQueryReceipt,
) -> RealRepoMarkdownKnowledgeSemanticScenarioQueryEvidenceReceipt {
    RealRepoMarkdownKnowledgeSemanticScenarioQueryEvidenceReceipt {
        query_id: query.query_id.clone(),
        query_kind: query.query_kind.clone(),
        query_ms: query.query_ms,
        passed: query.passed,
        required_top_path: query.required_top_path.clone(),
        observed_top_path: query.observed_top_path.clone(),
        missing_paths: query.missing_paths.clone(),
        observed_path_count: query.observed_paths.len(),
        failure_reason: (!query.passed).then(|| "query receipt failed".to_string()),
    }
}

fn missing_query_evidence(
    query_id: &str,
) -> RealRepoMarkdownKnowledgeSemanticScenarioQueryEvidenceReceipt {
    RealRepoMarkdownKnowledgeSemanticScenarioQueryEvidenceReceipt {
        query_id: query_id.to_string(),
        query_kind: "missing".to_string(),
        query_ms: 0,
        passed: false,
        required_top_path: None,
        observed_top_path: None,
        missing_paths: Vec::new(),
        observed_path_count: 0,
        failure_reason: Some("query receipt missing".to_string()),
    }
}

fn linked_semantic_gold_queries(
    gold_queries: &[RealRepoGoldQuery],
) -> Vec<(&RealRepoGoldQuery, &'static LinkedSemanticGoldQuery)> {
    let links_by_query_id = LINKED_SEMANTIC_GOLD_QUERIES
        .iter()
        .map(|link| (link.query_id, link))
        .collect::<BTreeMap<_, _>>();
    gold_queries
        .iter()
        .filter(|query| query.kind == RealRepoGoldQueryKind::LinkGraph)
        .filter_map(|query| {
            links_by_query_id
                .get(query.id.as_str())
                .map(|link| (query, *link))
        })
        .collect()
}

fn covered_markdown_paths_for_linked_queries(
    scope: &SemanticScopeBundle,
    linked_queries: &[(&RealRepoGoldQuery, &'static LinkedSemanticGoldQuery)],
) -> Result<Vec<String>, String> {
    let objects_by_id = scope
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();
    let mut covered = BTreeSet::new();

    for (query, link) in linked_queries {
        let object = objects_by_id.get(link.object_id).copied().ok_or_else(|| {
            format!(
                "semantic object `{}` missing for linked query `{}`",
                link.object_id, query.id
            )
        })?;
        for path in &query.must_hit_paths {
            if object_covers_markdown_path(object, path) {
                covered.insert(path.clone());
            }
        }
    }

    Ok(covered.into_iter().collect())
}

fn object_covers_markdown_path(object: &SemanticObject, path: &str) -> bool {
    if object.provenance.source == path {
        return true;
    }
    object
        .source_path
        .to_str()
        .is_some_and(|source_path| format!("semantic/{source_path}") == path)
}

fn required_semantic_relation_path_receipts()
-> Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt> {
    REQUIRED_SEMANTIC_RELATION_PATHS
        .iter()
        .map(relation_path_receipt)
        .collect()
}

fn covered_semantic_relation_paths(
    relations: &[SemanticRelationEdge],
) -> Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt> {
    relations
        .iter()
        .map(
            |relation| RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
                source: relation.source.clone(),
                kind: semantic_relation_kind_token(&relation.kind).to_string(),
                target: relation.target.clone(),
            },
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn semantic_knowledge_scenarios(
    linked_query_ids: &[String],
    semantic_object_ids: &[String],
    covered_relation_paths: &[RealRepoMarkdownKnowledgeSemanticRelationPathReceipt],
) -> Vec<RealRepoMarkdownKnowledgeSemanticScenarioReceipt> {
    let linked_query_id_set = linked_query_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let semantic_object_id_set = semantic_object_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let covered_relation_path_set = covered_relation_paths.iter().collect::<BTreeSet<_>>();

    REQUIRED_SEMANTIC_SCENARIOS
        .iter()
        .map(|scenario| {
            let linked_query_ids = scenario
                .linked_query_ids
                .iter()
                .map(|query_id| (*query_id).to_string())
                .collect::<Vec<_>>();
            let required_object_ids = scenario
                .required_object_ids
                .iter()
                .map(|object_id| (*object_id).to_string())
                .collect::<Vec<_>>();
            let covered_object_ids = scenario
                .required_object_ids
                .iter()
                .filter(|object_id| semantic_object_id_set.contains(**object_id))
                .map(|object_id| (*object_id).to_string())
                .collect::<Vec<_>>();
            let required_relation_paths = scenario
                .required_relation_paths
                .iter()
                .map(relation_path_receipt)
                .collect::<Vec<_>>();
            let covered_relation_paths = required_relation_paths
                .iter()
                .filter(|relation| covered_relation_path_set.contains(*relation))
                .cloned()
                .collect::<Vec<_>>();
            let queries_passed = scenario
                .linked_query_ids
                .iter()
                .all(|query_id| linked_query_id_set.contains(query_id));
            let objects_passed = covered_object_ids.len() == required_object_ids.len();
            let relations_passed = covered_relation_paths.len() == required_relation_paths.len();

            RealRepoMarkdownKnowledgeSemanticScenarioReceipt {
                scenario_id: scenario.scenario_id.to_string(),
                intent: scenario.intent.to_string(),
                linked_query_ids,
                query_evidence: Vec::new(),
                required_object_ids,
                covered_object_ids,
                required_relation_paths,
                covered_relation_paths,
                passed: queries_passed && objects_passed && relations_passed,
            }
        })
        .collect()
}

fn relation_path_receipt(
    path: &RequiredSemanticRelationPath,
) -> RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
        source: path.source.to_string(),
        kind: path.kind.to_string(),
        target: path.target.to_string(),
    }
}

fn relation_path_label(path: &RealRepoMarkdownKnowledgeSemanticRelationPathReceipt) -> String {
    format!("{}:{}:{}", path.source, path.kind, path.target)
}

fn semantic_relation_kind_token(kind: &SemanticRelationKind) -> &'static str {
    match kind {
        SemanticRelationKind::Contains => "contains",
        SemanticRelationKind::DependsOn => "depends_on",
        SemanticRelationKind::Constrains => "constrains",
        SemanticRelationKind::Implements => "implements",
        SemanticRelationKind::Governs => "governs",
        SemanticRelationKind::Affects => "affects",
        SemanticRelationKind::Validates => "validates",
        SemanticRelationKind::Supersedes => "supersedes",
        SemanticRelationKind::ProjectsTo => "projects_to",
        SemanticRelationKind::ConsumedBy => "consumed_by",
    }
}
