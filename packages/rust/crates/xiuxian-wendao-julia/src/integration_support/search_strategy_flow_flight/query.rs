use super::ids::repo_relative_source_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RepoSearchAttempt {
    pub(super) query: String,
    pub(super) path_prefix: String,
}

fn repo_search_query(source_path: &str, heading_anchor: Option<&str>) -> String {
    let Some(anchor) = heading_anchor else {
        return source_path.to_owned();
    };
    let terms = search_terms(anchor);
    let effective_terms = stage_stripped_terms(terms.as_slice());
    if effective_terms.is_empty() {
        source_path.to_owned()
    } else {
        effective_terms.join(" ")
    }
}

fn stage_stripped_terms(terms: &[String]) -> &[String] {
    if terms.first().is_none_or(|term| term != "stage") {
        return terms;
    }
    match terms.get(1) {
        Some(term) if term.chars().all(|character| character.is_ascii_digit()) => &terms[2..],
        Some(_) => &terms[1..],
        None => &[],
    }
}

pub(super) fn candidate_discovery_queries(intent: &str) -> Vec<RepoSearchAttempt> {
    let mut attempts = Vec::new();
    let trimmed = intent.trim();

    let terms = search_terms(trimmed);
    if terms.is_empty() {
        push_repo_search_attempt(&mut attempts, trimmed, "");
        return attempts;
    }

    let frontload_route_attempts = should_frontload_route_scoped_attempts(terms.as_slice());
    if frontload_route_attempts {
        push_required_evidence_candidate_attempts(&mut attempts, terms.as_slice());
        push_exact_anchor_candidate_attempts(&mut attempts, terms.as_slice());
        push_route_hint_candidate_attempts(&mut attempts, terms.as_slice());
        push_domain_surface_candidate_attempts(&mut attempts, terms.as_slice());
    }
    push_repo_search_attempt(&mut attempts, trimmed, "");
    push_repo_search_attempt(&mut attempts, terms.join(" ").as_str(), "");
    if !frontload_route_attempts {
        push_domain_surface_candidate_attempts(&mut attempts, terms.as_slice());
        push_required_evidence_candidate_attempts(&mut attempts, terms.as_slice());
        push_exact_anchor_candidate_attempts(&mut attempts, terms.as_slice());
        push_route_hint_candidate_attempts(&mut attempts, terms.as_slice());
    }
    for window_size in [4, 3, 2] {
        if terms.len() < window_size {
            continue;
        }
        for window in terms.windows(window_size) {
            push_repo_search_attempt(&mut attempts, window.join(" ").as_str(), "");
        }
    }
    for term in terms.iter().filter(|term| term.len() >= 4) {
        push_repo_search_attempt(&mut attempts, term.as_str(), "");
    }
    attempts.truncate(32);
    attempts
}

fn should_frontload_route_scoped_attempts(terms: &[String]) -> bool {
    has_all_terms(terms, &["search", "strategy", "flow"])
        && has_any_term(
            terms,
            &[
                "ownership",
                "authority",
                "boundary",
                "validation",
                "gate",
                "path",
                "relation",
                "page",
                "index",
                "link",
                "graph",
                "materialization",
            ],
        )
}

fn push_required_evidence_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_all_terms(terms, &["search", "strategy", "flow"]) {
        push_repo_search_attempt(
            attempts,
            "SearchStrategyFlow",
            "packages/rust/crates/xiuxian-wendao-julia/README.md",
        );
    }
    if has_any_term(terms, &["ownership", "authority", "boundary"]) {
        push_repo_search_attempt(attempts, "ownership boundary", "docs/rfcs");
    }
    if has_any_term(terms, &["validation", "gate", "path", "materialization"]) {
        push_repo_search_attempt(attempts, "validation path", "docs/testing");
    }
    if has_all_terms(terms, &["search", "strategy", "flow"])
        && has_any_term(
            terms,
            &[
                "ownership",
                "authority",
                "boundary",
                "validation",
                "gate",
                "path",
                "materialization",
            ],
        )
    {
        push_repo_search_attempt(
            attempts,
            "Search Strategy Flow Link Graph",
            "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy",
        );
    }
    if has_all_terms(terms, &["link", "graph"])
        || has_all_terms(terms, &["graph"])
        || has_all_terms(terms, &["relation"])
    {
        push_repo_search_attempt(
            attempts,
            "Search Strategy Flow Link Graph",
            "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy",
        );
    }
}

fn push_exact_anchor_candidate_attempts(attempts: &mut Vec<RepoSearchAttempt>, terms: &[String]) {
    if has_all_terms(terms, &["search", "strategy", "flow"]) {
        push_repo_search_attempt(attempts, "SearchStrategyFlow", "docs/30_search_strategy");
        push_repo_search_attempt(
            attempts,
            "SearchStrategyFlow",
            "packages/rust/crates/xiuxian-wendao-julia/docs",
        );
        push_repo_search_attempt(
            attempts,
            "SearchStrategyFlow",
            "packages/rust/crates/xiuxian-wendao-julia/README.md",
        );
        push_repo_search_attempt(attempts, "SearchStrategyFlow", "");
    }
    if has_all_terms(terms, &["page", "index"]) {
        push_repo_search_attempt(attempts, "PageIndex", "docs/20_page_index");
        push_repo_search_attempt(
            attempts,
            "PageIndex",
            "packages/rust/crates/xiuxian-wendao-julia/docs",
        );
        push_repo_search_attempt(attempts, "PageIndex", "");
    }
    if has_all_terms(terms, &["link", "graph"]) {
        push_repo_search_attempt(attempts, "LinkGraph", "docs/10_graph_compute");
        push_repo_search_attempt(attempts, "LinkGraph", "docs/01_core/wendao");
        push_repo_search_attempt(
            attempts,
            "LinkGraph",
            "packages/rust/crates/xiuxian-wendao-julia/docs",
        );
        push_repo_search_attempt(attempts, "LinkGraph", "");
    }
    if has_any_term(terms, &["ownership", "authority", "boundary"]) {
        push_repo_search_attempt(attempts, "ownership boundary", "docs/30_search_strategy");
        push_repo_search_attempt(
            attempts,
            "ownership boundary",
            "packages/rust/crates/xiuxian-wendao-julia/docs",
        );
        push_repo_search_attempt(attempts, "ownership boundary", "docs/rfcs");
    }
    if has_any_term(terms, &["validation", "gate", "path", "materialization"]) {
        push_repo_search_attempt(attempts, "validation path", "docs/90_validation");
        push_repo_search_attempt(
            attempts,
            "validation path",
            "packages/rust/crates/xiuxian-wendao-julia/docs",
        );
        push_repo_search_attempt(attempts, "validation path", "docs/testing");
        push_repo_search_attempt(
            attempts,
            "local validation CI test proof",
            "docs/developer/testing.md",
        );
    }
}

fn push_domain_surface_candidate_attempts(attempts: &mut Vec<RepoSearchAttempt>, terms: &[String]) {
    push_governance_surface_candidate_attempts(attempts, terms);
    push_attachment_surface_candidate_attempts(attempts, terms);
    push_flight_materialization_surface_candidate_attempts(attempts, terms);
    push_query_engine_surface_candidate_attempts(attempts, terms);
    push_memory_surface_candidate_attempts(attempts, terms);
    push_benchmark_surface_candidate_attempts(attempts, terms);
    push_code_adaptation_surface_candidate_attempts(attempts, terms);
    push_polyglot_surface_candidate_attempts(attempts, terms);
    push_projected_page_index_surface_candidate_attempts(attempts, terms);
}

fn push_governance_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_any_term(
        terms,
        &[
            "governance",
            "modularity",
            "modular",
            "warning",
            "warnings",
            "debt",
            "policy",
            "auditor",
            "agent",
            "agents",
        ],
    ) {
        push_repo_search_attempt(attempts, "modularity debt warning cleanup", "AGENTS.md");
        push_repo_search_attempt(attempts, "Debt Closure", "AGENTS.md");
        push_repo_search_attempt(
            attempts,
            "modularity debt warning cleanup",
            "docs/standards",
        );
        push_repo_search_attempt(
            attempts,
            "Hyper Modularity",
            "docs/standards/AUDITOR_CODEX.md",
        );
        push_repo_search_attempt(attempts, "governance modularity warnings", "docs/standards");
    }
}

fn push_attachment_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_any_term(
        terms,
        &[
            "attachment",
            "attachments",
            "analyzer",
            "docling",
            "ocr",
            "shard",
            "shards",
            "provenance",
        ],
    ) {
        push_repo_search_attempt(
            attempts,
            "Docling OCR shard provenance page index",
            "packages/rust/crates/xiuxian-wendao-attachments/README.md",
        );
        push_repo_search_attempt(
            attempts,
            "Docling OCR shard provenance page index",
            "packages/python/xiuxian-wendao-analyzer/README.md",
        );
        push_repo_search_attempt(
            attempts,
            "Docling structure OCR shard provenance",
            "packages/rust/crates/xiuxian-wendao-attachments",
        );
        push_repo_search_attempt(
            attempts,
            "Docling structure OCR shard provenance",
            "packages/python/xiuxian-wendao-analyzer",
        );
    }
}

fn push_flight_materialization_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_any_term(terms, &["studio", "flight", "materialization"]) {
        push_repo_search_attempt(
            attempts,
            "Studio SearchStrategyFlow Flight materialization ownership",
            "packages/rust/crates/xiuxian-wendao-studio/README.md",
        );
        push_repo_search_attempt(
            attempts,
            "SearchStrategyFlow Flight materialization bridge",
            "packages/rust/crates/xiuxian-wendao-julia/README.md",
        );
    }
}

fn push_query_engine_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_all_terms(terms, &["query", "engine"]) {
        push_repo_search_attempt(
            attempts,
            "Wendao query engine ownership boundary source authority",
            "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md",
        );
    }
}

fn push_memory_surface_candidate_attempts(attempts: &mut Vec<RepoSearchAttempt>, terms: &[String]) {
    if has_any_term(terms, &["memory", "working", "knowledge"])
        && has_any_term(terms, &["searchstrategyflow", "search", "strategy", "flow"])
    {
        push_repo_search_attempt(
            attempts,
            "validated SearchStrategyFlow working knowledge memory layer",
            "docs/rfcs/2026-04-05-wendao-memory-layer-boundaries-rfc.md",
        );
    }
}

fn push_benchmark_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_any_term(terms, &["benchmark", "profile", "contract"]) {
        push_repo_search_attempt(
            attempts,
            "SearchStrategyFlow frontier rows required evidence coverage",
            "packages/python/wendao-knowledge-retrieval-benchmark/docs/profile_contract.md",
        );
        push_repo_search_attempt(
            attempts,
            "SearchStrategyFlow benchmark architecture",
            "packages/python/wendao-knowledge-retrieval-benchmark/docs/architecture.md",
        );
    }
}

fn push_code_adaptation_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_all_terms(terms, &["link", "graph"]) && has_any_term(terms, &["code", "adaptation"]) {
        push_repo_search_attempt(
            attempts,
            "LinkGraph code adaptation graph search evidence",
            "docs/02_dev/standards/LINK_GRAPH_CODE_ADAPTATION.md",
        );
    }
}

fn push_polyglot_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_any_term(terms, &["polyglot", "orchestrator"]) {
        push_repo_search_attempt(
            attempts,
            "polyglot compute orchestrator boundary calibration",
            "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md",
        );
        push_repo_search_attempt(
            attempts,
            "polyglot compute orchestrator boundary calibration audit",
            "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-audit.md",
        );
    }
}

fn push_projected_page_index_surface_candidate_attempts(
    attempts: &mut Vec<RepoSearchAttempt>,
    terms: &[String],
) {
    if has_all_terms(terms, &["page", "index"])
        && has_any_term(
            terms,
            &[
                "projected",
                "projection",
                "retrieval",
                "enhancement",
                "roadmap",
            ],
        )
    {
        push_repo_search_attempt(
            attempts,
            "projected documentation pages graph enhanced retrieval",
            "packages/rust/crates/xiuxian-wendao/docs/06_roadmap",
        );
        push_repo_search_attempt(
            attempts,
            "projected documentation pages graph enhanced retrieval",
            "packages/rust/crates/xiuxian-wendao/docs/06_roadmap/403_document_projection_and_retrieval_enhancement.md",
        );
        push_repo_search_attempt(
            attempts,
            "page index projected documentation retrieval",
            "packages/python/wendao-knowledge-retrieval-benchmark",
        );
    }
}

fn push_route_hint_candidate_attempts(attempts: &mut Vec<RepoSearchAttempt>, terms: &[String]) {
    if has_all_terms(terms, &["search", "strategy"]) {
        push_repo_search_attempt(attempts, "search strategy flow", "docs/30_search_strategy");
    }
    if has_all_terms(terms, &["page", "index"]) || has_all_terms(terms, &["reasoning", "tree"]) {
        push_repo_search_attempt(attempts, "page index reasoning tree", "docs/20_page_index");
    }
    if has_all_terms(terms, &["link", "graph"])
        || has_all_terms(terms, &["graph"])
        || has_all_terms(terms, &["relation"])
    {
        push_repo_search_attempt(attempts, "link graph compute", "docs/10_graph_compute");
        push_repo_search_attempt(
            attempts,
            "Search Strategy Flow Link Graph",
            "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy",
        );
    }
}

fn has_all_terms(terms: &[String], needles: &[&str]) -> bool {
    needles
        .iter()
        .all(|needle| terms.iter().any(|term| term == needle))
}

fn has_any_term(terms: &[String], needles: &[&str]) -> bool {
    needles
        .iter()
        .any(|needle| terms.iter().any(|term| term == needle))
}

pub(super) fn repo_search_attempts_for_route(
    repo_id: &str,
    source_path: &str,
    heading_anchor: Option<&str>,
) -> Vec<RepoSearchAttempt> {
    let mut attempts = Vec::new();
    let mut relaxed_attempts = Vec::new();
    let repo_relative_source_path = repo_relative_source_path(repo_id, source_path);
    let anchor_query = repo_search_query(repo_relative_source_path.as_str(), heading_anchor);
    push_repo_search_attempt(
        &mut attempts,
        anchor_query.as_str(),
        repo_relative_source_path.as_str(),
    );
    push_repo_search_attempt(&mut relaxed_attempts, anchor_query.as_str(), "");

    for file_query in source_path_queries(repo_relative_source_path.as_str()) {
        push_repo_search_attempt(
            &mut attempts,
            file_query.as_str(),
            repo_relative_source_path.as_str(),
        );
        push_repo_search_attempt(&mut relaxed_attempts, file_query.as_str(), "");
    }

    push_repo_search_attempt(
        &mut relaxed_attempts,
        repo_relative_source_path.as_str(),
        "",
    );
    if repo_relative_source_path != source_path.trim().trim_matches('/') {
        push_repo_search_attempt(&mut relaxed_attempts, source_path, "");
    }
    attempts.extend(relaxed_attempts);
    attempts
}

fn push_repo_search_attempt(attempts: &mut Vec<RepoSearchAttempt>, query: &str, path_prefix: &str) {
    let query = query.trim();
    if query.is_empty() {
        return;
    }
    let attempt = RepoSearchAttempt {
        query: query.to_owned(),
        path_prefix: path_prefix.trim().to_owned(),
    };
    if !attempts.contains(&attempt) {
        attempts.push(attempt);
    }
}

fn source_path_queries(source_path: &str) -> Vec<String> {
    let file_name = source_path
        .trim()
        .trim_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(source_path)
        .rsplit_once('.')
        .map_or_else(|| source_path.trim(), |(stem, _)| stem);
    let terms = search_terms(file_name);
    if terms.is_empty() {
        return vec![source_path.trim().to_owned()];
    }

    let semantic_terms = terms
        .iter()
        .filter(|term| !term.chars().all(|character| character.is_ascii_digit()))
        .cloned()
        .collect::<Vec<_>>();
    let mut queries = Vec::new();
    let joined_terms = terms.join(" ");
    push_unique_query(&mut queries, &joined_terms);
    if !semantic_terms.is_empty() {
        let joined_semantic_terms = semantic_terms.join(" ");
        push_unique_query(&mut queries, &joined_semantic_terms);
        let compact = semantic_terms.join("");
        if compact.len() >= 4 {
            push_unique_query(&mut queries, &compact);
        }
    }
    queries
}

fn push_unique_query(queries: &mut Vec<String>, query: &str) {
    let query = query.trim();
    if !query.is_empty() && !queries.iter().any(|existing| existing == query) {
        queries.push(query.to_owned());
    }
}

fn search_terms(text: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut previous_was_lowercase = false;

    for character in text.chars() {
        if character.is_ascii_alphanumeric() {
            if previous_was_lowercase && character.is_ascii_uppercase() && !current.is_empty() {
                terms.push(std::mem::take(&mut current));
            }
            current.push(character.to_ascii_lowercase());
            previous_was_lowercase = character.is_ascii_lowercase();
        } else {
            if !current.is_empty() {
                terms.push(std::mem::take(&mut current));
            }
            previous_was_lowercase = false;
        }
    }
    if !current.is_empty() {
        terms.push(current);
    }

    terms.into_iter().filter(|term| term.len() >= 2).collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/query.rs"]
mod tests;
