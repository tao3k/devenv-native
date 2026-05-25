use std::path::PathBuf;

use crate::analyzers::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRef, RepositoryRefreshPolicy,
};
use crate::search::real_repo_precision::types::{
    RealRepoGoldQuery, RealRepoGoldQueryKind, RealRepoKnowledgeScenario,
    RealRepoKnowledgeScenarioAuthorityExpectation, RealRepoKnowledgeScenarioKind,
    RealRepoKnowledgeScenarioQueryVariant, RealRepoKnowledgeScenarioQueryVariantKind,
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt, RealRepoPrecisionCatalogEntry,
};

pub(crate) fn default_real_repo_precision_catalog() -> Vec<RealRepoPrecisionCatalogEntry> {
    vec![artisan_workshop_catalog_entry(), pi_wendao_catalog_entry()]
}

fn artisan_workshop_catalog_entry() -> RealRepoPrecisionCatalogEntry {
    RealRepoPrecisionCatalogEntry {
        repository: artisan_workshop_repository(),
        include_dirs: artisan_workshop_include_dirs(),
        excluded_dirs: standard_excluded_dirs(),
        gold_queries: artisan_workshop_gold_queries(),
        knowledge_scenarios: default_knowledge_scenarios(),
    }
}

fn artisan_workshop_repository() -> RegisteredRepository {
    RegisteredRepository {
        id: "xiuxian-artisan-workshop".to_string(),
        path: None,
        url: Some("https://github.com/tao3k/xiuxian-artisan-workshop.git".to_string()),
        git_ref: Some(RepositoryRef::Branch("main".to_string())),
        refresh: RepositoryRefreshPolicy::Manual,
        plugins: vec![RepositoryPluginConfig::Id("ast-grep".to_string())],
    }
}

fn artisan_workshop_include_dirs() -> Vec<String> {
    vec![
        "docs".to_string(),
        "semantic".to_string(),
        "packages/rust/crates/xiuxian-wendao".to_string(),
        "packages/rust/crates/xiuxian-wendao/src/link_graph".to_string(),
        "packages/rust/crates/xiuxian-julia-runtime".to_string(),
    ]
}

fn standard_excluded_dirs() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".cache".to_string(),
        ".data".to_string(),
        ".direnv".to_string(),
        ".run".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
    ]
}

fn artisan_workshop_gold_queries() -> Vec<RealRepoGoldQuery> {
    let mut queries = semantic_gold_queries();
    queries.extend(docs_gold_queries());
    queries.extend(repo_ast_gold_queries());
    queries
}

fn semantic_gold_queries() -> Vec<RealRepoGoldQuery> {
    let mut queries = semantic_object_gold_queries();
    queries.extend(semantic_relation_gold_queries());
    queries
}

fn semantic_object_gold_queries() -> Vec<RealRepoGoldQuery> {
    vec![
        RealRepoGoldQuery {
            id: "repo-native-semantic-ssot-rfc".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "repo-native semantic SSOT".to_string(),
            limit: 10,
            must_hit_paths: vec![
                "docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md".to_string(),
            ],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "wendao-page-index-reasoning".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "PageIndex reasoning tables".to_string(),
            limit: 10,
            must_hit_paths: vec!["packages/rust/crates/xiuxian-wendao/README.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "link-graph-ppr-algorithm".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "LinkGraph PPR Algorithm Spec".to_string(),
            limit: 10,
            must_hit_paths: vec!["docs/01_core/wendao/ppr-algorithm.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "semantic-object-wendao-query-substrate".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "component.wendao.query-substrate Wendao Query Substrate".to_string(),
            limit: 10,
            must_hit_paths: vec![
                "semantic/objects/component/wendao-query-substrate.md".to_string(),
            ],
            required_top_path: Some(
                "semantic/objects/component/wendao-query-substrate.md".to_string(),
            ),
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "semantic-decision-repo-native-authority".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "decision.semantic-ssot.repo-native-first Repo-Native Semantic Authority First"
                .to_string(),
            limit: 10,
            must_hit_paths: vec![
                "semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string(),
            ],
            required_top_path: Some(
                "semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string(),
            ),
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "semantic-decision-projections-read-models".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "decision.semantic-ssot.projections-are-read-models Projections Are Read Models"
                .to_string(),
            limit: 10,
            must_hit_paths: vec![
                "semantic/objects/decision/semantic-ssot-projections-are-read-models.md"
                    .to_string(),
            ],
            required_top_path: Some(
                "semantic/objects/decision/semantic-ssot-projections-are-read-models.md"
                    .to_string(),
            ),
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "semantic-invariant-llm-output-not-authority".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "invariant.llm-output-is-not-authority LLM Output Is Not Authority".to_string(),
            limit: 10,
            must_hit_paths: vec![
                "semantic/objects/invariant/llm-output-is-not-authority.md".to_string(),
            ],
            required_top_path: Some(
                "semantic/objects/invariant/llm-output-is-not-authority.md".to_string(),
            ),
            language_filters: Vec::new(),
        },
    ]
}

fn semantic_relation_gold_queries() -> Vec<RealRepoGoldQuery> {
    vec![
        RealRepoGoldQuery {
            id: "semantic-relation-repo-native-governs-query-substrate".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query:
                    "decision.semantic-ssot.repo-native-first Repo-Native Semantic Authority First governs component.wendao.query-substrate"
                        .to_string(),
                limit: 10,
                must_hit_paths: vec![
                    "semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string(),
                ],
                required_top_path: Some(
                    "semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string(),
                ),
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "semantic-relation-projections-govern-llm-boundary".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query:
                    "decision.semantic-ssot.projections-are-read-models Projections Are Read Models governs invariant.llm-output-is-not-authority"
                        .to_string(),
                limit: 10,
                must_hit_paths: vec![
                    "semantic/objects/decision/semantic-ssot-projections-are-read-models.md"
                        .to_string(),
                ],
                required_top_path: Some(
                    "semantic/objects/decision/semantic-ssot-projections-are-read-models.md"
                        .to_string(),
                ),
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "semantic-relation-llm-constrains-projections".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query:
                    "invariant.llm-output-is-not-authority LLM Output Is Not Authority constrains decision.semantic-ssot.projections-are-read-models"
                        .to_string(),
                limit: 10,
                must_hit_paths: vec![
                    "semantic/objects/invariant/llm-output-is-not-authority.md".to_string(),
                ],
                required_top_path: Some(
                    "semantic/objects/invariant/llm-output-is-not-authority.md".to_string(),
                ),
                language_filters: Vec::new(),
            },
    ]
}

fn docs_gold_queries() -> Vec<RealRepoGoldQuery> {
    let mut queries = docs_primary_gold_queries();
    queries.extend(docs_boundary_gold_queries());
    queries
}

fn docs_primary_gold_queries() -> Vec<RealRepoGoldQuery> {
    vec![
        RealRepoGoldQuery {
            id: "docs-documentation-hierarchy-standard".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "Filesystem-Based Documentation Hierarchy DFS-2026".to_string(),
            limit: 20,
            must_hit_paths: vec!["docs/02_dev/standards/DOCUMENTATION_HIERARCHY.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "docs-documentation-hierarchy-standard-paraphrase".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "Where is the canonical filesystem documentation hierarchy standard?"
                .to_string(),
            limit: 20,
            must_hit_paths: vec!["docs/02_dev/standards/DOCUMENTATION_HIERARCHY.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "docs-wendao-agentic-retrieval".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "Wendao Agentic Retrieval Autonomous Query Planning".to_string(),
            limit: 20,
            must_hit_paths: vec!["docs/03_features/wendao-agentic-retrieval.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "docs-wendao-agentic-retrieval-paraphrase".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "How does Wendao help an agent plan and expand knowledge retrieval?".to_string(),
            limit: 20,
            must_hit_paths: vec!["docs/03_features/wendao-agentic-retrieval.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "docs-memory-architecture".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "Omni-Memory Self-Evolving Memory Engine Wendao Memory Layer Boundaries"
                .to_string(),
            limit: 20,
            must_hit_paths: vec!["docs/01_core/memory/architecture.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
        RealRepoGoldQuery {
            id: "docs-llm-routing-guide".to_string(),
            kind: RealRepoGoldQueryKind::LinkGraph,
            query: "LLM Routing Guide routing confidence score".to_string(),
            limit: 20,
            must_hit_paths: vec!["docs/99_llm/routing-guide.md".to_string()],
            required_top_path: None,
            language_filters: Vec::new(),
        },
    ]
}

fn docs_boundary_gold_queries() -> Vec<RealRepoGoldQuery> {
    vec![
        RealRepoGoldQuery {
            id: "docs-polyglot-compute-orchestrator-rfc".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "Polyglot Compute Orchestrator Rust Python Julia boundary calibration"
                    .to_string(),
                limit: 20,
                must_hit_paths: vec![
                    "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md".to_string(),
                ],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "docs-polyglot-page-index-agent-task".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query:
                    "Agent task change WendaoGraph PageIndex boundary polyglot orchestrator RFC PageIndex reasoning tables"
                        .to_string(),
                limit: 20,
                must_hit_paths: vec![
                    "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md".to_string(),
                    "packages/rust/crates/xiuxian-wendao/README.md".to_string(),
                ],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "docs-wendao-memory-layer-boundaries-rfc".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "Wendao Memory Layer Boundaries runtime cache episodic memory durable knowledge"
                    .to_string(),
                limit: 20,
                must_hit_paths: vec![
                    "docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md".to_string(),
                ],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "docs-wendao-context-snapshot".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "Wendao ContextSnap Stateful Context Governance".to_string(),
                limit: 20,
                must_hit_paths: vec!["docs/03_features/wendao-context-snapshot.md".to_string()],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "docs-wendao-context-snapshot-alias".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "ContextSnap context snapshot governance stateful context".to_string(),
                limit: 20,
                must_hit_paths: vec!["docs/03_features/wendao-context-snapshot.md".to_string()],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "docs-traceability-policy".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "TRACEABILITY_POLICY Engineering Traceability Policy Digital Thread HMAS Taxonomy"
                    .to_string(),
                limit: 20,
                must_hit_paths: vec!["docs/02_dev/standards/TRACEABILITY_POLICY.md".to_string()],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "docs-root-index-registry".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "CyberXiuXian Project Registry DFS-2026".to_string(),
                limit: 20,
                must_hit_paths: vec!["docs/00_vision/ROOT_INDEX.md".to_string()],
                required_top_path: None,
                language_filters: Vec::new(),
            },
    ]
}

fn repo_ast_gold_queries() -> Vec<RealRepoGoldQuery> {
    vec![
        RealRepoGoldQuery {
            id: "repo-source-materialization-function".to_string(),
            kind: RealRepoGoldQueryKind::RepoAst,
            query: "resolve_registered_repository_source".to_string(),
            limit: 10,
            must_hit_paths: vec![
                "packages/rust/crates/xiuxian-wendao/src/analyzers/repo_source.rs".to_string(),
            ],
            required_top_path: Some(
                "packages/rust/crates/xiuxian-wendao/src/analyzers/repo_source.rs".to_string(),
            ),
            language_filters: vec!["rust".to_string()],
        },
        RealRepoGoldQuery {
            id: "repo-code-search-outcome-struct".to_string(),
            kind: RealRepoGoldQueryKind::RepoAst,
            query: "RepoCodeSearchOutcome".to_string(),
            limit: 10,
            must_hit_paths: vec![
                "packages/rust/crates/xiuxian-wendao/src/search/repo_search/orchestration.rs"
                    .to_string(),
            ],
            required_top_path: Some(
                "packages/rust/crates/xiuxian-wendao/src/search/repo_search/orchestration.rs"
                    .to_string(),
            ),
            language_filters: vec!["rust".to_string()],
        },
        RealRepoGoldQuery {
            id: "repo-code-search-query-async-function".to_string(),
            kind: RealRepoGoldQueryKind::RepoAst,
            query: "search_repo_code_outcome_for_query".to_string(),
            limit: 10,
            must_hit_paths: vec![
                "packages/rust/crates/xiuxian-wendao/src/search/repo_search/orchestration.rs"
                    .to_string(),
            ],
            required_top_path: Some(
                "packages/rust/crates/xiuxian-wendao/src/search/repo_search/orchestration.rs"
                    .to_string(),
            ),
            language_filters: vec!["rust".to_string()],
        },
        RealRepoGoldQuery {
            id: "repo-link-graph-build-with-filters-source".to_string(),
            kind: RealRepoGoldQueryKind::RepoAst,
            query: "build_with_filters".to_string(),
            limit: 10,
            must_hit_paths: vec![
                "packages/rust/crates/xiuxian-wendao/src/link_graph/index/build/assemble/api.rs"
                    .to_string(),
            ],
            required_top_path: Some(
                "packages/rust/crates/xiuxian-wendao/src/link_graph/index/build/assemble/api.rs"
                    .to_string(),
            ),
            language_filters: vec!["rust".to_string()],
        },
    ]
}

fn pi_wendao_catalog_entry() -> RealRepoPrecisionCatalogEntry {
    RealRepoPrecisionCatalogEntry {
        repository: RegisteredRepository {
            id: "pi-wendao".to_string(),
            path: Some(PathBuf::from(".data/pi-wendao")),
            url: Some("https://github.com/tao3k/pi-wendao.git".to_string()),
            git_ref: None,
            refresh: RepositoryRefreshPolicy::Manual,
            plugins: vec![RepositoryPluginConfig::Id("ast-grep".to_string())],
        },
        include_dirs: vec![".".to_string()],
        excluded_dirs: vec![
            ".git".to_string(),
            "dist".to_string(),
            "node_modules".to_string(),
            "coverage".to_string(),
        ],
        gold_queries: vec![
            RealRepoGoldQuery {
                id: "pi-wendao-readme-subagents-host".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query:
                    "pi-subagents host execution qianji checkpoint parallel scheduling graph trace"
                        .to_string(),
                limit: 20,
                must_hit_paths: vec!["README.md".to_string()],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "pi-wendao-named-workflows-brainstorm-cache".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "named workflows brainstorm PRJ_CACHE_HOME canonical seed qianji scheduling"
                    .to_string(),
                limit: 20,
                must_hit_paths: vec!["docs/named-workflows.md".to_string()],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "pi-wendao-bpmn-format-runtime-ownership".to_string(),
                kind: RealRepoGoldQueryKind::LinkGraph,
                query: "BPMN qianji owns scheduling checkpoints pi-wendao renders human prompts"
                    .to_string(),
                limit: 20,
                must_hit_paths: vec!["docs/bpmn-format.md".to_string()],
                required_top_path: None,
                language_filters: Vec::new(),
            },
            RealRepoGoldQuery {
                id: "pi-wendao-subagents-extension-source".to_string(),
                kind: RealRepoGoldQueryKind::RepoAst,
                query: "createCliPiSubagentsHost".to_string(),
                limit: 10,
                must_hit_paths: vec!["src/cli/pi-subagents.ts".to_string()],
                required_top_path: Some("src/cli/pi-subagents.ts".to_string()),
                language_filters: vec!["typescript".to_string()],
            },
            RealRepoGoldQuery {
                id: "pi-wendao-agent-host-interface-source".to_string(),
                kind: RealRepoGoldQueryKind::RepoAst,
                query: "buildPiWendaoAgentPrompt".to_string(),
                limit: 10,
                must_hit_paths: vec!["src/executor/agent-host.ts".to_string()],
                required_top_path: Some("src/executor/agent-host.ts".to_string()),
                language_filters: vec!["typescript".to_string()],
            },
            RealRepoGoldQuery {
                id: "pi-wendao-model-resolver-source".to_string(),
                kind: RealRepoGoldQueryKind::RepoAst,
                query: "resolveModel".to_string(),
                limit: 10,
                must_hit_paths: vec!["src/cli/model-resolver.ts".to_string()],
                required_top_path: Some("src/cli/model-resolver.ts".to_string()),
                language_filters: vec!["typescript".to_string()],
            },
        ],
        knowledge_scenarios: pi_wendao_knowledge_scenarios(),
    }
}

fn pi_wendao_knowledge_scenarios() -> Vec<RealRepoKnowledgeScenario> {
    vec![
        RealRepoKnowledgeScenario {
            id: "pi-wendao-agent-workflow-boundary".to_string(),
            kind: RealRepoKnowledgeScenarioKind::AgentTask,
            intent: "Gather evidence for how pi-wendao owns subagent/workflow orchestration while qianji owns BPMN scheduling.".to_string(),
            linked_query_ids: vec![
                "pi-wendao-readme-subagents-host".to_string(),
                "pi-wendao-bpmn-format-runtime-ownership".to_string(),
            ],
            query_variants: query_variants(&[
                (
                    "pi-wendao-readme-subagents-host",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "pi-wendao-bpmn-format-runtime-ownership",
                    RealRepoKnowledgeScenarioQueryVariantKind::Task,
                ),
            ]),
            required_paths: vec!["README.md".to_string(), "docs/bpmn-format.md".to_string()],
            required_semantic_object_ids: Vec::new(),
            required_relation_paths: Vec::new(),
            authority: None,
            forbidden_paths: Vec::new(),
        },
        RealRepoKnowledgeScenario {
            id: "pi-wendao-named-workflow-entrypoint".to_string(),
            kind: RealRepoKnowledgeScenarioKind::KnownItem,
            intent: "Find the named workflow documentation for the native brainstorm entrypoint.".to_string(),
            linked_query_ids: vec!["pi-wendao-named-workflows-brainstorm-cache".to_string()],
            query_variants: query_variants(&[(
                "pi-wendao-named-workflows-brainstorm-cache",
                RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
            )]),
            required_paths: vec!["docs/named-workflows.md".to_string()],
            required_semantic_object_ids: Vec::new(),
            required_relation_paths: Vec::new(),
            authority: None,
            forbidden_paths: Vec::new(),
        },
    ]
}

fn default_knowledge_scenarios() -> Vec<RealRepoKnowledgeScenario> {
    let mut scenarios = core_knowledge_scenarios();
    scenarios.extend(semantic_knowledge_scenarios());
    scenarios.extend(boundary_knowledge_scenarios());
    scenarios
}

fn core_knowledge_scenarios() -> Vec<RealRepoKnowledgeScenario> {
    vec![
        RealRepoKnowledgeScenario {
            id: "known-item-documentation-hierarchy".to_string(),
            kind: RealRepoKnowledgeScenarioKind::KnownItem,
            intent: "Find the canonical documentation hierarchy standard.".to_string(),
            linked_query_ids: vec!["docs-documentation-hierarchy-standard".to_string()],
            query_variants: query_variants(&[(
                "docs-documentation-hierarchy-standard",
                RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
            ), (
                "docs-documentation-hierarchy-standard-paraphrase",
                RealRepoKnowledgeScenarioQueryVariantKind::Paraphrase,
            )]),
            required_paths: vec!["docs/02_dev/standards/DOCUMENTATION_HIERARCHY.md".to_string()],
            required_semantic_object_ids: Vec::new(),
            required_relation_paths: Vec::new(),
            authority: None,
            forbidden_paths: Vec::new(),
        },
        RealRepoKnowledgeScenario {
            id: "natural-language-agentic-retrieval".to_string(),
            kind: RealRepoKnowledgeScenarioKind::NaturalLanguageIntent,
            intent: "Answer a natural-language question about how Wendao supports autonomous agent retrieval.".to_string(),
            linked_query_ids: vec!["docs-wendao-agentic-retrieval".to_string()],
            query_variants: query_variants(&[
                (
                    "docs-wendao-agentic-retrieval",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "docs-wendao-agentic-retrieval-paraphrase",
                    RealRepoKnowledgeScenarioQueryVariantKind::Paraphrase,
                ),
            ]),
            required_paths: vec!["docs/03_features/wendao-agentic-retrieval.md".to_string()],
            required_semantic_object_ids: Vec::new(),
            required_relation_paths: Vec::new(),
            authority: None,
            forbidden_paths: Vec::new(),
        },
    ]
}

fn semantic_knowledge_scenarios() -> Vec<RealRepoKnowledgeScenario> {
    vec![
        RealRepoKnowledgeScenario {
            id: "multi-hop-projection-authority-boundary".to_string(),
            kind: RealRepoKnowledgeScenarioKind::MultiHopRelation,
            intent: "Explain why projections and LLM output are read-model evidence rather than semantic authority.".to_string(),
            linked_query_ids: vec![
                "semantic-decision-projections-read-models".to_string(),
                "semantic-invariant-llm-output-not-authority".to_string(),
                "semantic-relation-projections-govern-llm-boundary".to_string(),
                "semantic-relation-llm-constrains-projections".to_string(),
            ],
            query_variants: query_variants(&[
                (
                    "semantic-decision-projections-read-models",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "semantic-invariant-llm-output-not-authority",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "semantic-relation-projections-govern-llm-boundary",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "semantic-relation-llm-constrains-projections",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
            ]),
            required_paths: vec![
                "semantic/objects/decision/semantic-ssot-projections-are-read-models.md"
                    .to_string(),
                "semantic/objects/invariant/llm-output-is-not-authority.md".to_string(),
            ],
            required_semantic_object_ids: vec![
                "decision.semantic-ssot.projections-are-read-models".to_string(),
                "invariant.llm-output-is-not-authority".to_string(),
            ],
            required_relation_paths: vec![
                relation_path(
                    "decision.semantic-ssot.projections-are-read-models",
                    "governs",
                    "invariant.llm-output-is-not-authority",
                ),
                relation_path(
                    "invariant.llm-output-is-not-authority",
                    "constrains",
                    "decision.semantic-ssot.projections-are-read-models",
                ),
            ],
            authority: None,
            forbidden_paths: Vec::new(),
        },
        RealRepoKnowledgeScenario {
            id: "authority-repo-native-semantic-ssot".to_string(),
            kind: RealRepoKnowledgeScenarioKind::AuthorityOrdering,
            intent: "Prefer the repo-native semantic authority decision over broad package documentation when resolving SSOT ownership.".to_string(),
            linked_query_ids: vec!["semantic-decision-repo-native-authority".to_string()],
            query_variants: query_variants(&[(
                "semantic-decision-repo-native-authority",
                RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
            )]),
            required_paths: vec![
                "semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string(),
            ],
            required_semantic_object_ids: vec![
                "decision.semantic-ssot.repo-native-first".to_string(),
            ],
            required_relation_paths: vec![relation_path(
                "decision.semantic-ssot.repo-native-first",
                "governs",
                "component.wendao.query-substrate",
            )],
            authority: Some(RealRepoKnowledgeScenarioAuthorityExpectation {
                preferred_path:
                    "semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string(),
                competing_paths: vec!["packages/rust/crates/xiuxian-wendao/README.md".to_string()],
            }),
            forbidden_paths: Vec::new(),
        },
        RealRepoKnowledgeScenario {
            id: "negative-llm-output-authority-guard".to_string(),
            kind: RealRepoKnowledgeScenarioKind::NegativeEvidence,
            intent: "Guard against treating LLM output as semantic authority by retrieving the explicit invariant.".to_string(),
            linked_query_ids: vec!["semantic-invariant-llm-output-not-authority".to_string()],
            query_variants: query_variants(&[(
                "semantic-invariant-llm-output-not-authority",
                RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
            )]),
            required_paths: vec![
                "semantic/objects/invariant/llm-output-is-not-authority.md".to_string(),
            ],
            required_semantic_object_ids: vec![
                "invariant.llm-output-is-not-authority".to_string(),
            ],
            required_relation_paths: Vec::new(),
            authority: None,
            forbidden_paths: vec!["packages/rust/crates/xiuxian-wendao/README.md".to_string()],
        },
    ]
}

fn boundary_knowledge_scenarios() -> Vec<RealRepoKnowledgeScenario> {
    vec![
        RealRepoKnowledgeScenario {
            id: "ambiguous-alias-contextsnap".to_string(),
            kind: RealRepoKnowledgeScenarioKind::AmbiguousAlias,
            intent: "Resolve the ContextSnap naming alias to the canonical stateful context governance document.".to_string(),
            linked_query_ids: vec!["docs-wendao-context-snapshot".to_string()],
            query_variants: query_variants(&[
                (
                    "docs-wendao-context-snapshot",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "docs-wendao-context-snapshot-alias",
                    RealRepoKnowledgeScenarioQueryVariantKind::Alias,
                ),
            ]),
            required_paths: vec!["docs/03_features/wendao-context-snapshot.md".to_string()],
            required_semantic_object_ids: Vec::new(),
            required_relation_paths: Vec::new(),
            authority: None,
            forbidden_paths: Vec::new(),
        },
        RealRepoKnowledgeScenario {
            id: "agent-task-polyglot-page-index-boundary".to_string(),
            kind: RealRepoKnowledgeScenarioKind::AgentTask,
            intent: "Gather the evidence an agent needs before changing the polyglot PageIndex/WendaoGraph boundary.".to_string(),
            linked_query_ids: vec![
                "docs-polyglot-compute-orchestrator-rfc".to_string(),
                "wendao-page-index-reasoning".to_string(),
            ],
            query_variants: query_variants(&[
                (
                    "docs-polyglot-compute-orchestrator-rfc",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "wendao-page-index-reasoning",
                    RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
                ),
                (
                    "docs-polyglot-page-index-agent-task",
                    RealRepoKnowledgeScenarioQueryVariantKind::Task,
                ),
            ]),
            required_paths: vec![
                "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md".to_string(),
                "packages/rust/crates/xiuxian-wendao/README.md".to_string(),
            ],
            required_semantic_object_ids: Vec::new(),
            required_relation_paths: Vec::new(),
            authority: None,
            forbidden_paths: Vec::new(),
        },
    ]
}

fn relation_path(
    source: &str,
    kind: &str,
    target: &str,
) -> RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
        source: source.to_string(),
        kind: kind.to_string(),
        target: target.to_string(),
    }
}

fn query_variants(
    variants: &[(&str, RealRepoKnowledgeScenarioQueryVariantKind)],
) -> Vec<RealRepoKnowledgeScenarioQueryVariant> {
    variants
        .iter()
        .map(|(query_id, kind)| RealRepoKnowledgeScenarioQueryVariant {
            query_id: (*query_id).to_string(),
            kind: *kind,
        })
        .collect()
}
