//! Host-process probes for local `WendaoGraph.jl` contracts.

use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use super::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInputBatch, search_strategy_flow_candidate_input_batch_from_markdown,
};
use super::search_strategy_flow_flight::{
    SearchStrategyFlowFlightMaterializationConfig, materialize_search_strategy_flow_routes,
    search_strategy_flow_candidate_input_batch_from_repo_search,
};
use super::service_runtime::repo_root;

const WENDAOGRAPH_PACKAGE_DIR_ENV: &str = "WENDAOGRAPH_PACKAGE_DIR";
const WENDAOGRAPH_JULIA_PROJECT_ENV: &str = "WENDAOGRAPH_JULIA_PROJECT";
const WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV: &str = "WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS";
const WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES_ENV: &str =
    "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES";
const WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES_ENV: &str = "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT";
const WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS_ENV: &str =
    "WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS";
const PAGE_INDEX_HOST_PROBE_PREFIX: &str = "wendaograph_page_index_host_probe";
const LINK_GRAPH_HOST_PROBE_PREFIX: &str = "wendaograph_link_graph_host_probe";

const SEARCH_STRATEGY_FLOW_JULIA: &str = r#"
using WendaoGraph

intent = ARGS[1]
search_root = ARGS[2]
candidate_input_tsv = length(ARGS) >= 3 ? ARGS[3] : ""
candidate_input_source_hint = length(ARGS) >= 4 ? ARGS[4] : ""

function json_escape(value)
    text = string(value)
    text = replace(text, "\\" => "\\\\")
    text = replace(text, "\"" => "\\\"")
    text = replace(text, "\n" => "\\n")
    text = replace(text, "\r" => "\\r")
    text = replace(text, "\t" => "\\t")
    "\"$text\""
end

function json_value(value)
    if value isa AbstractString
        return json_escape(value)
    elseif value isa Bool
        return value ? "true" : "false"
    elseif value isa Integer || value isa AbstractFloat
        return string(value)
    elseif value isa AbstractVector
        return "[" * join(json_value.(value), ",") * "]"
    end
    json_escape(value)
end

function json_object(pairs)
    "{" * join(["$(json_escape(first(pair))):$(json_value(last(pair)))" for pair in pairs], ",") * "}"
end

function json_pair(name, raw_value)
    "$(json_escape(name)):$(raw_value)"
end

function split_candidate_id(candidate_id)
    parts = split(candidate_id, Char(0x23); limit=2)
    relative_path = String(parts[1])
    heading_anchor = length(parts) == 2 ? String(parts[2]) : ""
    relative_path, heading_anchor
end

function real_doc_path(relative_path)
    joinpath(search_root, split(relative_path, '/')...)
end

function markdown_anchor(title)
    text = lowercase(strip(replace(title, Char(0x60) => "")))
    text = replace(text, r"[^a-z0-9]+" => "-")
    text = replace(text, r"^-+" => "")
    text = replace(text, r"-+$" => "")
    isempty(text) ? "section" : text
end

function markdown_section_text(candidate_id)
    relative_path, heading_anchor = split_candidate_id(candidate_id)
    path = real_doc_path(relative_path)
    isfile(path) || return ""
    text = read(path, String)
    isempty(heading_anchor) && return text

    lines = split(text, '\n'; keepempty=true)
    heading_pattern = r"^(#{1,6})\s+(.+?)\s*$"
    selected = String[]
    in_section = false
    section_level = 0

    for line in lines
        matched = match(heading_pattern, line)
        if matched !== nothing
            level = length(matched.captures[1])
            title = strip(matched.captures[2])
            if in_section && level <= section_level
                break
            end
            if !in_section && markdown_anchor(title) == heading_anchor
                in_section = true
                section_level = level
                push!(selected, line)
                continue
            end
        end
        in_section && push!(selected, line)
    end

    join(selected, "\n")
end

function doc_context_cost(candidate_id)
    text = markdown_section_text(candidate_id)
    isempty(text) && return 512
    max(1, ceil(Int, sizeof(text) / 20))
end

function unescape_candidate_field(value)
    buffer = IOBuffer()
    escaped = false
    for char in value
        if escaped
            if char == 't'
                print(buffer, '\t')
            elseif char == 'n'
                print(buffer, '\n')
            elseif char == 'r'
                print(buffer, '\r')
            elseif char == '\\'
                print(buffer, '\\')
            else
                print(buffer, char)
            end
            escaped = false
        elseif char == '\\'
            escaped = true
        else
            print(buffer, char)
        end
    end
    escaped && print(buffer, '\\')
    String(take!(buffer))
end

function parse_candidate_bool(value)
    normalized = lowercase(strip(value))
    normalized == "true" && return true
    normalized == "false" && return false
    error("invalid SearchStrategyFlow candidate bool: $value")
end

function parse_candidate_inputs(payload)
    rows = NamedTuple[]
    isempty(strip(payload)) && return rows

    for (line_number, line) in enumerate(split(payload, '\n'; keepempty=false))
        fields = split(line, '\t'; keepempty=true)
        length(fields) == 13 || error("candidate input TSV row $line_number expected 13 fields, got $(length(fields))")
        decoded = unescape_candidate_field.(fields)
        edge_kinds = isempty(decoded[13]) ? String[] : String.(split(decoded[13], ','; keepempty=false))
        push!(
            rows,
            (
                relative_path = decoded[1],
                heading_anchor = decoded[2],
                title = decoded[3],
                line_start = parse(Int, decoded[4]),
                line_end = parse(Int, decoded[5]),
                context_cost = parse(Int, decoded[6]),
                evidence_coverage = parse(Float64, decoded[7]),
                graph_score = parse(Float64, decoded[8]),
                authority_score = parse(Float64, decoded[9]),
                structural_score = parse(Float64, decoded[10]),
                uncertainty = parse(Float64, decoded[11]),
                blocked = parse_candidate_bool(decoded[12]),
                edge_kinds = edge_kinds,
            ),
        )
    end

    rows
end

function doc_candidate(relative_path, heading_anchor; evidence_coverage, graph_score, authority_score, structural_score, uncertainty, blocked=false, edge_kinds=("anchor", "linkography", "authority"), context_cost_override=nothing)
    candidate_id = "$(relative_path)#$(heading_anchor)"
    context_cost_value = context_cost_override === nothing ? doc_context_cost(candidate_id) : context_cost_override
    (
        candidate_id = candidate_id,
        candidate_kind = "markdown_heading_section",
        node_ids = ["intent", relative_path, candidate_id, "markdown-section", "package-docs"],
        edge_kinds = collect(edge_kinds),
        evidence_coverage = evidence_coverage,
        graph_score = graph_score,
        authority_score = authority_score,
        semantic_score = 0.0,
        structural_score = structural_score,
        context_cost = context_cost_value,
        uncertainty = uncertainty,
        blocked = blocked,
    )
end

function doc_candidate_from_input(input)
    doc_candidate(
        input.relative_path,
        input.heading_anchor;
        evidence_coverage = input.evidence_coverage,
        graph_score = input.graph_score,
        authority_score = input.authority_score,
        structural_score = input.structural_score,
        uncertainty = input.uncertainty,
        blocked = input.blocked,
        edge_kinds = input.edge_kinds,
        context_cost_override = input.context_cost,
    )
end

flow_id = "pi-wendao-search-strategy-flow"
query_understanding = query_understanding_evidence_rows(intent; flow_id = flow_id, intent_id = "cli-intent-1")
strategy_budget = (
    source = isempty(query_understanding) ? "default" : "query_understanding",
    loop_budget = isempty(query_understanding) ? 1 : maximum(row.recommended_loop_budget for row in query_understanding),
    judgement_budget = isempty(query_understanding) ? 1 : maximum(row.recommended_judgement_budget for row in query_understanding),
    beam_width = isempty(query_understanding) ? 3 : maximum(row.recommended_beam_width for row in query_understanding),
)

normalized_intent = lowercase(intent)
strategy_weight = occursin("strategy", normalized_intent) || occursin("search", normalized_intent) || occursin("flow", normalized_intent) ? 0.04 : 0.0
page_index_weight = occursin("page", normalized_intent) || occursin("index", normalized_intent) ? 0.03 : 0.0

function fixed_proof_candidates()
    [
        doc_candidate(
            "docs/30_search_strategy/30.01_search_strategy_flow.md",
            "stage-1-query-understanding";
            evidence_coverage = min(1.0, 0.94 + strategy_weight),
            graph_score = min(1.0, 0.91 + strategy_weight),
            authority_score = 0.93,
            structural_score = 0.90,
            uncertainty = 0.09,
            edge_kinds = ("anchor", "search-strategy", "authority", "page-index"),
        ),
        doc_candidate(
            "docs/20_page_index/20.01_reasoning_tree_contracts.md",
            "relationship-to-search-strategy";
            evidence_coverage = min(1.0, 0.76 + page_index_weight),
            graph_score = min(1.0, 0.80 + page_index_weight),
            authority_score = 0.84,
            structural_score = 0.88,
            uncertainty = 0.22,
            edge_kinds = ("anchor", "page-index", "evidence-plane"),
        ),
        doc_candidate(
            "docs/10_graph_compute/10.01_link_graph_compute.md",
            "how-this-helps-linkgraph-search";
            evidence_coverage = 0.63,
            graph_score = 0.79,
            authority_score = 0.70,
            structural_score = 0.66,
            uncertainty = 0.34,
            edge_kinds = ("linkography", "graph-compute", "supporting-evidence"),
        ),
        doc_candidate(
            "docs/90_validation/90.01_validation.md",
            "promotion-boundary";
            evidence_coverage = 0.74,
            graph_score = 0.65,
            authority_score = 0.82,
            structural_score = 0.72,
            uncertainty = 0.18,
            blocked = true,
            edge_kinds = ("validation", "negative-guard"),
        ),
    ]
end

candidate_inputs = parse_candidate_inputs(candidate_input_tsv)
candidate_input_source = isempty(candidate_inputs) ? "fixed-proof-fallback" : (isempty(candidate_input_source_hint) ? "rust-markdown-headings" : candidate_input_source_hint)
candidates = isempty(candidate_inputs) ? fixed_proof_candidates() : doc_candidate_from_input.(candidate_inputs)

rows = strategy_flow_candidate_rows(
    candidates;
    flow_id = flow_id,
    revision_id = "query-graph-cli-1",
    keep_threshold = 0.70,
    expand_threshold = 0.45,
    context_budget = 4096,
    query_understanding = query_understanding,
)
transitions = strategy_flow_transition_rows(rows; flow_id = flow_id)
frontier = strategy_flow_frontier_rows(
    rows;
    flow_id = flow_id,
    beam_width = strategy_budget.beam_width,
    context_budget = 1900,
)
actions = strategy_flow_planner_action_rows(
    rows,
    transitions,
    frontier;
    flow_id = flow_id,
    loop_budget = strategy_budget.loop_budget,
    judgement_budget = strategy_budget.judgement_budget,
    compare_count = 1,
)

function query_understanding_json(row)
    json_object((
        "flowId" => row.flow_id,
        "intentId" => row.intent_id,
        "signalId" => row.signal_id,
        "signalKind" => row.signal_kind,
        "signalValue" => row.signal_value,
        "confidence" => row.confidence,
        "routeHint" => row.route_hint,
        "requiredEvidence" => row.required_evidence,
        "ambiguity" => row.ambiguity,
        "weight" => row.weight,
        "recommendedLoopBudget" => row.recommended_loop_budget,
        "recommendedJudgementBudget" => row.recommended_judgement_budget,
        "recommendedBeamWidth" => row.recommended_beam_width,
        "reason" => row.reason,
    ))
end

function strategy_budget_json(row)
    json_object((
        "source" => row.source,
        "loopBudget" => row.loop_budget,
        "judgementBudget" => row.judgement_budget,
        "beamWidth" => row.beam_width,
    ))
end

function candidate_json(row)
    json_object((
        "candidateId" => row.candidate_id,
        "action" => row.action,
        "reason" => row.reason,
        "finalScore" => row.final_score,
        "evidenceCoverage" => row.evidence_coverage,
        "graphScore" => row.graph_score,
        "authorityScore" => row.authority_score,
        "semanticScore" => row.semantic_score,
        "structuralScore" => row.structural_score,
        "contextCost" => row.context_cost,
        "blocked" => row.blocked,
    ))
end

function frontier_json(row)
    json_object((
        "candidateId" => row.candidate_id,
        "rank" => row.rank,
        "selected" => row.selected,
        "finalScore" => row.final_score,
        "action" => row.action,
        "contextBudget" => row.context_budget,
        "judgementKind" => row.judgement_kind,
    ))
end

function action_json(row)
    json_object((
        "actionKind" => row.action_kind,
        "candidateId" => row.candidate_id,
        "targetCandidateId" => row.target_candidate_id,
        "cycleAllowed" => row.cycle_allowed,
        "requiresLlmJudgement" => row.requires_llm_judgement,
        "score" => row.score,
        "contextBudget" => row.context_budget,
        "reason" => row.reason,
    ))
end

function stage_receipt_json(row)
    json_object((
        "stage" => row.stage,
        "notebook" => row.notebook,
        "inputCount" => row.input_count,
        "outputCount" => row.output_count,
        "selectedCount" => row.selected_count,
        "llmJudgementCount" => row.llm_judgement_count,
        "cycleAllowedCount" => row.cycle_allowed_count,
        "contextBudget" => row.context_budget,
        "summary" => row.summary,
    ))
end

total_context = sum(row.context_cost for row in rows)
selected_context = sum(row.context_budget for row in frontier)
selected_ids = [row.candidate_id for row in frontier if row.selected]
llm_action_count = count(row -> row.requires_llm_judgement, actions)
cycle_action_count = count(row -> row.cycle_allowed, actions)
stage_receipts = [
    (
        stage = "query_understanding",
        notebook = "notebooks/search_strategy_flow_query_understanding.jl",
        input_count = 1,
        output_count = length(query_understanding),
        selected_count = 0,
        llm_judgement_count = 0,
        cycle_allowed_count = 0,
        context_budget = 0,
        summary = "intent to graph route hints, required evidence, ambiguity, and strategy budget",
    ),
    (
        stage = "candidate_scoring",
        notebook = "notebooks/search_strategy_flow_candidate_scoring.jl",
        input_count = length(candidates),
        output_count = length(rows),
        selected_count = count(row -> row.action != "prune", rows),
        llm_judgement_count = 0,
        cycle_allowed_count = 0,
        context_budget = total_context,
        summary = "graph evidence rows to deterministic score rows and branch actions",
    ),
    (
        stage = "transition_inference",
        notebook = "notebooks/search_strategy_flow_transition_inference.jl",
        input_count = length(rows),
        output_count = length(transitions),
        selected_count = count(row -> row.transition_kind != "stop_branch", transitions),
        llm_judgement_count = 0,
        cycle_allowed_count = 0,
        context_budget = 0,
        summary = "score rows to revision transition kinds and missing-signal diagnostics",
    ),
    (
        stage = "frontier_selection",
        notebook = "notebooks/search_strategy_flow_frontier_selection.jl",
        input_count = length(rows),
        output_count = length(frontier),
        selected_count = length(selected_ids),
        llm_judgement_count = count(row -> row.selected && row.judgement_kind == "subagent_branch_judgement", frontier),
        cycle_allowed_count = 0,
        context_budget = selected_context,
        summary = "beam and context-budget bounded Agent-visible frontier",
    ),
    (
        stage = "planner_actions",
        notebook = "notebooks/search_strategy_flow_planner_actions.jl",
        input_count = length(frontier),
        output_count = length(actions),
        selected_count = count(row -> row.action_kind != "stop", actions),
        llm_judgement_count = llm_action_count,
        cycle_allowed_count = cycle_action_count,
        context_budget = sum(row.context_budget for row in actions),
        summary = "frontier and transition facts to materialize, refine, judge, compare, and stop actions",
    ),
]
summary = json_object((
    "candidateCount" => length(rows),
    "selectedCount" => length(selected_ids),
    "plannerActionCount" => length(actions),
    "totalContextCost" => total_context,
    "selectedContextCost" => selected_context,
    "contextReductionRatio" => 1.0 - selected_context / total_context,
))
validation = json_object((
    "noVectorMode" => all(row.semantic_score == 0.0 for row in rows),
    "materializedTopCandidate" => any(row.action_kind == "materialize" && row.candidate_id in selected_ids for row in actions),
    "blockedEvidencePruned" => all(row -> !row.blocked || !(row.candidate_id in selected_ids), rows),
    "selectedContextReduced" => selected_context < total_context,
))

println("{" * join((
    json_pair("intent", json_value(intent)),
    json_pair("backend", json_value("rust-wendao-julia")),
    json_pair("controlPlane", json_value("rust")),
    json_pair("juliaProject", json_value(Base.active_project() === nothing ? "" : dirname(Base.active_project()))),
    json_pair("graphProject", json_value(Base.active_project() === nothing ? "" : dirname(Base.active_project()))),
    json_pair("searchRoot", json_value(search_root)),
    json_pair("candidateInputSource", json_value(candidate_input_source)),
    json_pair("candidateInputCount", json_value(length(candidate_inputs))),
    json_pair("queryUnderstanding", "[" * join(query_understanding_json.(query_understanding), ",") * "]"),
    json_pair("strategyBudget", strategy_budget_json(strategy_budget)),
    json_pair("stageReceipts", "[" * join(stage_receipt_json.(stage_receipts), ",") * "]"),
    json_pair("candidates", "[" * join(candidate_json.(rows), ",") * "]"),
    json_pair("frontier", "[" * join(frontier_json.(frontier), ",") * "]"),
    json_pair("plannerActions", "[" * join(action_json.(actions), ",") * "]"),
    json_pair("summary", summary),
    json_pair("validation", validation),
), ",") * "}")
"#;

const PAGE_INDEX_HOST_PROBE_JULIA: &str = r#"
using WendaoGraph

function tsv_rows(path)
    lines = split(read(path, String), '\n')
    isempty(lines) && return String[], Vector{Vector{String}}()
    header = split(first(lines), '\t'; keepempty = true)
    rows = Vector{Vector{String}}()
    for line in lines[2:end]
        isempty(strip(line)) && continue
        push!(rows, split(line, '\t'; keepempty = true))
    end

    header, rows
end

function require_header(header, expected, subject)
    String.(header) == collect(String.(expected)) ||
        error("$subject header mismatch")
end

function page_index_nodes_from_fixture(fixture_dir)
    header, rows = tsv_rows(joinpath(fixture_dir, "page_index_nodes.tsv"))
    require_header(header, page_index_node_columns(), "page_index_nodes")
    page_index_node_columntable([
        (
            node_id = row[1],
            page_id = row[2],
            parent_id = row[3],
            depth = parse(Int, row[4]),
            rank = parse(Int, row[5]),
            title = row[6],
            summary = row[7],
            line_start = parse(Int, row[8]),
            line_end = parse(Int, row[9]),
            token_count = parse(Int, row[10]),
        ) for row in rows
    ])
end

function page_index_edges_from_fixture(fixture_dir)
    header, rows = tsv_rows(joinpath(fixture_dir, "page_index_edges.tsv"))
    require_header(header, page_index_edge_columns(), "page_index_edges")
    page_index_edge_columntable([
        (
            source_id = row[1],
            target_id = row[2],
            edge_kind = row[3],
            weight = parse(Float64, row[4]),
        ) for row in rows
    ])
end

function page_index_seeds_from_fixture(fixture_dir)
    header, rows = tsv_rows(joinpath(fixture_dir, "page_index_seeds.tsv"))
    require_header(header, page_index_seed_columns(), "page_index_seeds")
    page_index_seed_columntable([
        (node_id = row[1], weight = parse(Float64, row[2]), seed_kind = row[3]) for
        row in rows
    ])
end

function timed_request(request)
    started = time_ns()
    result = page_index_reasoning_from_request(
        request;
        max_depth = 1,
        fanout = 1,
        tree_id = "host-probe",
    )
    elapsed_ms = (time_ns() - started) / 1_000_000
    elapsed_ms, result
end

function percentile(sorted_samples, ratio)
    index = clamp(ceil(Int, length(sorted_samples) * ratio), 1, length(sorted_samples))
    sorted_samples[index]
end

function truthy_env(name)
    lowercase(get(ENV, name, "0")) in ("1", "true", "yes", "on")
end

function planner_action_counts(request, result)
    actions = page_index_planner_action_table(
        result.reasoning_frontier;
        node_ids = request.page_index_nodes.node_id,
        jump_targets = ["docs/beta#beta"],
        stop_threshold = 1.0,
    )
    validate_page_index_planner_action_table(
        actions;
        frontier = result.reasoning_frontier,
        node_ids = request.page_index_nodes.node_id,
    )
    kind_counts = Dict("expand" => 0, "compare" => 0, "jump" => 0, "stop" => 0)
    for action_kind in actions.action_kind
        kind = String(action_kind)
        kind_counts[kind] = get(kind_counts, kind, 0) + 1
    end

    (
        rows = length(actions.action_id),
        expand = kind_counts["expand"],
        compare = kind_counts["compare"],
        jump = kind_counts["jump"],
        stop = kind_counts["stop"],
    )
end

function render_probe_report()
    fixture_dir = ENV["WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE"]
    sample_count = max(
        parse(Int, get(ENV, "WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES", "3")),
        1,
    )
    request = (
        page_index_nodes = page_index_nodes_from_fixture(fixture_dir),
        page_index_edges = page_index_edges_from_fixture(fixture_dir),
        page_index_seeds = page_index_seeds_from_fixture(fixture_dir),
    )

    first_ms, first_result = timed_request(request)
    validate_page_index_reasoning_tables(first_result)
    frontier_rows = length(first_result.reasoning_frontier.node_id)
    trace_rows = length(first_result.disclosure_trace.step_id)
    action_counts = truthy_env("WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS") ?
                    planner_action_counts(request, first_result) :
                    (rows = 0, expand = 0, compare = 0, jump = 0, stop = 0)

    samples = Float64[]
    for _ in 1:sample_count
        elapsed_ms, result = timed_request(request)
        validate_page_index_reasoning_tables(result)
        length(result.reasoning_frontier.node_id) == frontier_rows ||
            error("frontier row count changed")
        length(result.disclosure_trace.step_id) == trace_rows ||
            error("trace row count changed")
        if truthy_env("WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS")
            planner_action_counts(request, result) == action_counts ||
                error("planner action counts changed")
        end
        push!(samples, elapsed_ms)
    end
    sorted_samples = sort(samples)

    println(
        "wendaograph_page_index_host_probe " *
        "sample_count=$(sample_count) " *
        "first_ms=$(round(first_ms; digits = 3)) " *
        "warm_min_ms=$(round(sorted_samples[begin]; digits = 3)) " *
        "warm_median_ms=$(round(percentile(sorted_samples, 0.5); digits = 3)) " *
        "warm_p95_ms=$(round(percentile(sorted_samples, 0.95); digits = 3)) " *
        "warm_max_ms=$(round(last(sorted_samples); digits = 3)) " *
        "frontier_rows=$(frontier_rows) " *
        "trace_rows=$(trace_rows) " *
        "planner_action_rows=$(action_counts.rows) " *
        "planner_expand_actions=$(action_counts.expand) " *
        "planner_compare_actions=$(action_counts.compare) " *
        "planner_jump_actions=$(action_counts.jump) " *
        "planner_stop_actions=$(action_counts.stop)",
    )
end

render_probe_report()
"#;

const LINK_GRAPH_HOST_PROBE_JULIA: &str = r#"
using WendaoGraph

function request_roots(request)
    hasproperty(request, :seeds) && length(request.seeds.node_id) > 0 &&
        return [String(request.seeds.node_id[1])]
    [String(request.nodes.id[1])]
end

function timed_request(request)
    started = time_ns()
    result = link_graph_evidence_from_request(
        request;
        component_kinds = :weak,
        hnsw_bidirectional = true,
        max_depth = 1,
        fanout = 2,
        roots = request_roots(request),
        tree_id = "host-probe",
    )
    elapsed_ms = (time_ns() - started) / 1_000_000
    elapsed_ms, result
end

function percentile(sorted_samples, ratio)
    index = clamp(ceil(Int, length(sorted_samples) * ratio), 1, length(sorted_samples))
    sorted_samples[index]
end

function base_link_graph_request()
    (
        nodes = (id = ["alpha", "beta", "gamma", "delta"],),
        edges = (source_id = ["alpha", "beta"], target_id = ["beta", "gamma"]),
        seeds = diffusion_seed_columntable([diffusion_seed_row("alpha")]),
    )
end

function semantic_neighbor_request()
    base = base_link_graph_request()
    merge(base, (semantic_neighbors = semantic_neighbor_columntable([(
        query_id = "alpha",
        neighbor_id = "delta",
        query_index = 1,
        neighbor_index = 4,
        rank = 1,
        distance = 0.0,
    ),]),))
end

function semantic_overlay_request()
    base = base_link_graph_request()
    merge(base, (semantic_overlay = semantic_overlay_columntable([
        (
            source_id = "alpha",
            target_id = "delta",
            source_index = 1,
            target_index = 4,
            rank = 1,
            distance = 0.0,
            weight = 1.0,
            edge_kind = "semantic",
        ),
        (
            source_id = "delta",
            target_id = "alpha",
            source_index = 4,
            target_index = 1,
            rank = 1,
            distance = 0.0,
            weight = 1.0,
            edge_kind = "semantic",
        ),
    ]),))
end

function env_int(name, default_value)
    max(parse(Int, get(ENV, name, string(default_value))), 1)
end

function synthetic_large_request()
    node_count = max(env_int("WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES", 256), 4)
    fanout = min(
        max(env_int("WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT", 4), 1),
        node_count - 1,
    )
    semantic_neighbor_count = min(
        max(env_int("WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS", node_count), 1),
        node_count,
    )
    ids = ["node_$(index)" for index in 1:node_count]
    sources = String[]
    targets = String[]
    for source_index in 1:node_count
        for offset in 1:fanout
            push!(sources, ids[source_index])
            push!(targets, ids[((source_index + offset - 1) % node_count) + 1])
        end
    end
    semantic_neighbors = [
        (
            query_id = ids[1],
            neighbor_id = ids[index],
            query_index = 1,
            neighbor_index = index,
            rank = index,
            distance = Float64(index - 1) / max(semantic_neighbor_count, 1),
        ) for index in 1:semantic_neighbor_count
    ]

    (
        nodes = (id = ids,),
        edges = (source_id = sources, target_id = targets),
        seeds = diffusion_seed_columntable([diffusion_seed_row(ids[1])]),
        semantic_neighbors = semantic_neighbor_columntable(semantic_neighbors),
    )
end

function request_node_count(request)
    length(request.nodes.id)
end

function request_edge_count(request)
    length(request.edges.source_id)
end

function request_semantic_neighbor_count(request)
    hasproperty(request, :semantic_neighbors) && return length(request.semantic_neighbors.query_id)
    hasproperty(request, :semantic_overlay) && return length(request.semantic_overlay.source_id)
    0
end

function link_graph_probe_request(mode)
    mode == "semantic-neighbors" && return semantic_neighbor_request()
    mode == "semantic-overlay" && return semantic_overlay_request()
    mode == "synthetic-large" && return synthetic_large_request()
    error("unsupported WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE=$(mode)")
end

function render_probe_report()
    sample_count = max(
        parse(Int, get(ENV, "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES", "3")),
        1,
    )
    mode = get(ENV, "WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_MODE", "semantic-neighbors")
    request = link_graph_probe_request(mode)
    node_count = request_node_count(request)
    edge_count = request_edge_count(request)
    semantic_neighbor_count = request_semantic_neighbor_count(request)

    first_ms, first_result = timed_request(request)
    first_counts = validate_link_graph_evidence_tables(first_result)

    samples = Float64[]
    for _ in 1:sample_count
        elapsed_ms, result = timed_request(request)
        validate_link_graph_evidence_tables(result) == first_counts ||
            error("LinkGraph evidence row counts changed")
        push!(samples, elapsed_ms)
    end
    sorted_samples = sort(samples)

    println(
        "wendaograph_link_graph_host_probe " *
        "mode=$(mode) " *
        "node_count=$(node_count) " *
        "edge_count=$(edge_count) " *
        "semantic_neighbor_count=$(semantic_neighbor_count) " *
        "sample_count=$(sample_count) " *
        "first_ms=$(round(first_ms; digits = 3)) " *
        "warm_min_ms=$(round(sorted_samples[begin]; digits = 3)) " *
        "warm_median_ms=$(round(percentile(sorted_samples, 0.5); digits = 3)) " *
        "warm_p95_ms=$(round(percentile(sorted_samples, 0.95); digits = 3)) " *
        "warm_max_ms=$(round(last(sorted_samples); digits = 3)) " *
        "graph_metric_rows=$(first_counts.graph_metrics) " *
        "component_rows=$(first_counts.components) " *
        "topology_profile_rows=$(first_counts.topology_profile) " *
        "topology_candidate_rows=$(first_counts.topology_candidates) " *
        "topology_bottleneck_rows=$(first_counts.topology_bottlenecks) " *
        "topology_community_rows=$(first_counts.topology_communities) " *
        "topology_cover_rows=$(first_counts.topology_cover) " *
        "topology_core_rows=$(first_counts.topology_core) " *
        "topology_boundary_rows=$(first_counts.topology_boundary) " *
        "topology_transition_rows=$(first_counts.topology_transitions) " *
        "topology_gateway_rows=$(first_counts.topology_gateways) " *
        "topology_community_summary_rows=$(first_counts.topology_community_summaries) " *
        "topology_community_link_rows=$(first_counts.topology_community_links) " *
        "topology_community_frontier_rows=$(first_counts.topology_community_frontier) " *
        "semantic_overlay_rows=$(first_counts.semantic_overlay) " *
        "diffusion_rows=$(first_counts.diffusion_scores) " *
        "frontier_rows=$(first_counts.link_frontier)",
    )
end

render_probe_report()
"#;

/// Timing report from one local `WendaoGraph.jl` `PageIndex` host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphPageIndexHostProbeReport {
    /// Number of warm samples measured after the first request.
    pub sample_count: usize,
    /// First host-request call elapsed milliseconds after Julia package load.
    pub first_ms: f64,
    /// Minimum warm-call elapsed milliseconds.
    pub warm_min_ms: f64,
    /// Median warm-call elapsed milliseconds.
    pub warm_median_ms: f64,
    /// P95 warm-call elapsed milliseconds.
    pub warm_p95_ms: f64,
    /// Maximum warm-call elapsed milliseconds.
    pub warm_max_ms: f64,
    /// Reasoning frontier row count returned by the Julia facade.
    pub frontier_rows: usize,
    /// Disclosure trace row count returned by the Julia facade.
    pub trace_rows: usize,
}

/// Timing and action-count report from one local `WendaoGraph.jl` `PageIndex`
/// planner-action host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphPageIndexPlannerActionHostProbeReport {
    /// Base `PageIndex` host-probe timing and row-count report.
    pub base: WendaoGraphPageIndexHostProbeReport,
    /// Planner action row count returned by the Julia facade.
    pub planner_action_rows: usize,
    /// Number of expand actions.
    pub planner_expand_actions: usize,
    /// Number of compare actions.
    pub planner_compare_actions: usize,
    /// Number of jump actions.
    pub planner_jump_actions: usize,
    /// Number of stop actions.
    pub planner_stop_actions: usize,
}

/// Timing report from one local `WendaoGraph.jl` `LinkGraph` host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphLinkGraphHostProbeReport {
    /// Probe input mode selected for the host-process request.
    pub mode: String,
    /// Number of input graph nodes.
    pub node_count: usize,
    /// Number of input graph edges.
    pub edge_count: usize,
    /// Number of semantic-neighbor or semantic-overlay input rows.
    pub semantic_neighbor_count: usize,
    /// Number of warm samples measured after the first request.
    pub sample_count: usize,
    /// First host-request call elapsed milliseconds after Julia package load.
    pub first_ms: f64,
    /// Minimum warm-call elapsed milliseconds.
    pub warm_min_ms: f64,
    /// Median warm-call elapsed milliseconds.
    pub warm_median_ms: f64,
    /// P95 warm-call elapsed milliseconds.
    pub warm_p95_ms: f64,
    /// Maximum warm-call elapsed milliseconds.
    pub warm_max_ms: f64,
    /// Graph metric row count returned by the Julia facade.
    pub graph_metric_rows: usize,
    /// Topology candidate row count returned by the Julia facade.
    pub topology_candidate_rows: usize,
    /// Semantic overlay row count returned by the Julia facade.
    pub semantic_overlay_rows: usize,
    /// Diffusion score row count returned by the Julia facade.
    pub diffusion_rows: usize,
    /// Link frontier row count returned by the Julia facade.
    pub frontier_rows: usize,
}

/// Timing report plus full structural row counts from one local
/// `WendaoGraph.jl` `LinkGraph` host probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoGraphLinkGraphFullStructuralHostProbeReport {
    /// Base `LinkGraph` host-probe timing and core row-count report.
    pub base: WendaoGraphLinkGraphHostProbeReport,
    /// Component row count returned by the Julia facade.
    pub component_rows: usize,
    /// Topology profile row count returned by the Julia facade.
    pub topology_profile_rows: usize,
    /// Topology bottleneck row count returned by the Julia facade.
    pub topology_bottleneck_rows: usize,
    /// Topology community row count returned by the Julia facade.
    pub topology_community_rows: usize,
    /// Topology cover row count returned by the Julia facade.
    pub topology_cover_rows: usize,
    /// Topology core row count returned by the Julia facade.
    pub topology_core_rows: usize,
    /// Topology boundary row count returned by the Julia facade.
    pub topology_boundary_rows: usize,
    /// Topology transition row count returned by the Julia facade.
    pub topology_transition_rows: usize,
    /// Topology gateway row count returned by the Julia facade.
    pub topology_gateway_rows: usize,
    /// Topology community summary row count returned by the Julia facade.
    pub topology_community_summary_rows: usize,
    /// Topology community link row count returned by the Julia facade.
    pub topology_community_link_rows: usize,
    /// Topology community frontier row count returned by the Julia facade.
    pub topology_community_frontier_rows: usize,
}

/// Adds Rust-owned `SearchStrategyFlow` retrieval-route plans to a
/// `WendaoGraph.jl` trace.
///
/// The Julia side remains the owner of query understanding, graph scoring,
/// frontier pruning, and planner actions. This helper derives the Studio
/// Flight route contract from selected/planned candidates so downstream
/// `pi-wendao` execution can consume a single bridge trace without treating a
/// local fixture or static row count as executed materialization.
///
/// # Errors
///
/// Returns an error when the supplied trace is not valid JSON, the JSON root is
/// not an object, or the enriched trace cannot be serialized.
pub fn enrich_wendaograph_search_strategy_flow_retrieval_routes(
    trace: &str,
) -> Result<String, String> {
    let mut value = serde_json::from_str::<Value>(trace)
        .map_err(|error| format!("parse WendaoGraph SearchStrategyFlow JSON trace: {error}"))?;
    add_search_strategy_flow_retrieval_routes(&mut value)?;
    serialize_search_strategy_flow_trace(&value)
}

/// Adds Rust-owned `SearchStrategyFlow` retrieval-route plans to a trace, then
/// executes them through a real Arrow Flight endpoint.
///
/// # Errors
///
/// Returns an error when the supplied trace is invalid JSON, route enrichment
/// fails, the endpoint cannot be reached, or a route cannot be decoded into
/// evidence receipts.
pub async fn enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
    trace: &str,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<String, String> {
    let mut value = serde_json::from_str::<Value>(trace)
        .map_err(|error| format!("parse WendaoGraph SearchStrategyFlow JSON trace: {error}"))?;
    add_search_strategy_flow_retrieval_routes(&mut value)?;
    materialize_search_strategy_flow_routes(&mut value, config).await?;
    serialize_search_strategy_flow_trace(&value)
}

/// Runs local `WendaoGraph.jl` `SearchStrategyFlow` through the Rust owner
/// bridge and returns the JSON trace emitted by Julia.
///
/// # Errors
///
/// Returns an error when the intent is blank, the local `WendaoGraph.jl`
/// project or search root cannot be resolved, the Julia process exits
/// unsuccessfully, or the trace is not valid JSON.
pub fn run_wendaograph_search_strategy_flow_json(
    intent: &str,
    search_root: impl Into<PathBuf>,
) -> Result<String, String> {
    let trace = run_wendaograph_search_strategy_flow_raw_json(intent, search_root)?;
    enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace)
}

/// Runs local `WendaoGraph.jl` `SearchStrategyFlow` through the Rust owner
/// bridge, optionally executes the planned native Flight route sequence, and
/// returns the JSON trace emitted by Julia.
///
/// # Errors
///
/// Returns an error when the Julia host request fails, route enrichment fails,
/// or configured Flight materialization cannot decode route evidence.
pub async fn run_wendaograph_search_strategy_flow_json_with_flight_materialization(
    intent: &str,
    search_root: impl Into<PathBuf>,
    config: Option<SearchStrategyFlowFlightMaterializationConfig>,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;
    let search_root = search_root.into();
    let trace = match config.as_ref() {
        Some(config) => {
            let candidate_batch =
                search_strategy_flow_candidate_input_batch_from_repo_search(intent, config).await?;
            run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
                intent,
                search_root.as_path(),
                candidate_batch,
            )?
        }
        None => run_wendaograph_search_strategy_flow_raw_json(intent, search_root.as_path())?,
    };
    match config {
        Some(config) => {
            enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
                &trace, &config,
            )
            .await
        }
        None => enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace),
    }
}

fn run_wendaograph_search_strategy_flow_raw_json(
    intent: &str,
    search_root: impl Into<PathBuf>,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;
    let search_root = search_root.into();
    let search_root =
        resolve_existing_path("WendaoGraph SearchStrategyFlow search root", search_root)?;
    let candidate_batch =
        search_strategy_flow_candidate_input_batch_from_markdown(intent, search_root.as_path())?;
    run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
        intent,
        search_root.as_path(),
        candidate_batch,
    )
}

fn run_wendaograph_search_strategy_flow_raw_json_with_candidate_batch(
    intent: &str,
    search_root: impl Into<PathBuf>,
    candidate_batch: SearchStrategyFlowCandidateInputBatch,
) -> Result<String, String> {
    validate_search_strategy_flow_intent(intent)?;

    let julia_project = wendaograph_julia_project()?;
    let search_root =
        resolve_existing_path("WendaoGraph SearchStrategyFlow search root", search_root)?;
    debug_assert_eq!(
        candidate_batch.row_count,
        candidate_batch
            .tsv
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    );
    let julia_command = env::var("JULIA").unwrap_or_else(|_| "julia".to_owned());
    let output = Command::new(julia_command)
        .arg(format!("--project={}", julia_project.display()))
        .arg("--startup-file=no")
        .arg("-e")
        .arg(SEARCH_STRATEGY_FLOW_JULIA)
        .arg(intent)
        .arg(search_root)
        .arg(candidate_batch.tsv)
        .arg(candidate_batch.source)
        .output()
        .map_err(|error| format!("spawn WendaoGraph SearchStrategyFlow host request: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph SearchStrategyFlow host request exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trace = stdout.trim();
    if trace.is_empty() {
        return Err("WendaoGraph SearchStrategyFlow host request returned empty output".to_owned());
    }
    Ok(trace.to_owned())
}

fn validate_search_strategy_flow_intent(intent: &str) -> Result<(), String> {
    if intent.trim().is_empty() {
        return Err("SearchStrategyFlow intent must not be blank".to_owned());
    }
    Ok(())
}

fn add_search_strategy_flow_retrieval_routes(value: &mut Value) -> Result<(), String> {
    let routes = build_search_strategy_flow_retrieval_routes(value);
    let object = value.as_object_mut().ok_or_else(|| {
        "WendaoGraph SearchStrategyFlow JSON trace root must be an object".to_owned()
    })?;
    object.insert("retrievalRoutes".to_owned(), Value::Array(routes));
    Ok(())
}

fn serialize_search_strategy_flow_trace(value: &Value) -> Result<String, String> {
    serde_json::to_string(value)
        .map(|trace| format!("{trace}\n"))
        .map_err(|error| format!("serialize enriched SearchStrategyFlow JSON trace: {error}"))
}

fn build_search_strategy_flow_retrieval_routes(trace: &Value) -> Vec<Value> {
    let selected_candidate_ids = trace
        .get("frontier")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| json_bool(row, "selected"))
        .filter_map(|row| json_string(row, "candidateId"))
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();

    let action_candidate_ids = trace
        .get("plannerActions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| json_string(row, "actionKind") != Some("stop"))
        .flat_map(|row| {
            [
                json_string(row, "candidateId"),
                json_string(row, "targetCandidateId"),
            ]
        })
        .flatten()
        .filter(|id| !id.is_empty())
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>();

    trace
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|candidate| !json_bool(candidate, "blocked"))
        .filter_map(|candidate| {
            let candidate_id = json_string(candidate, "candidateId")?;
            (selected_candidate_ids.contains(candidate_id)
                || action_candidate_ids.contains(candidate_id))
            .then_some(candidate_id)
        })
        .map(search_strategy_flow_retrieval_route)
        .collect()
}

fn search_strategy_flow_retrieval_route(candidate_id: &str) -> Value {
    let section = parse_markdown_section_candidate_id(candidate_id);
    let mut route = json!({
        "candidateId": candidate_id,
        "materializationOwner": "studio-rust",
        "materializationStatus": "planned",
        "receiptSource": "rust-bridge",
        "primaryTransport": "arrow-flight",
        "sourcePath": section.source_path,
        "directFileReadAllowed": false,
        "executeBeforeAnswer": true,
        "flightSteps": search_strategy_flow_flight_steps(&section),
    });
    if let (Some(object), Some(heading_anchor)) = (route.as_object_mut(), section.heading_anchor) {
        object.insert("headingAnchor".to_owned(), json!(heading_anchor));
    }
    route
}

struct MarkdownSectionCandidate<'a> {
    source_path: &'a str,
    heading_anchor: Option<&'a str>,
}

fn parse_markdown_section_candidate_id(candidate_id: &str) -> MarkdownSectionCandidate<'_> {
    let (source_path, heading_anchor) = candidate_id.split_once('#').map_or(
        (candidate_id, None),
        |(source_path, heading_anchor)| {
            (
                source_path,
                (!heading_anchor.is_empty()).then_some(heading_anchor),
            )
        },
    );
    MarkdownSectionCandidate {
        source_path,
        heading_anchor,
    }
}

fn search_strategy_flow_flight_steps(section: &MarkdownSectionCandidate<'_>) -> Vec<Value> {
    let query = match section.heading_anchor {
        Some(heading_anchor) => format!("{}#{heading_anchor}", section.source_path),
        None => section.source_path.to_owned(),
    };
    let mut page_index_metadata = vec![
        "x-wendao-repo-projected-page-index-tree-repo=<repo>".to_owned(),
        "x-wendao-repo-projected-page-index-tree-page-id=<resolved-page-id>".to_owned(),
    ];
    if let Some(heading_anchor) = section.heading_anchor {
        page_index_metadata.push(format!("candidate-heading-anchor={heading_anchor}"));
    }

    vec![
        json!({
            "step": "flight_search_page",
            "transport": "arrow-flight",
            "route": "/search/repos/main",
            "metadataTemplates": [
                "x-wendao-repo-search-repo=<repo>",
                format!("x-wendao-repo-search-query={query}"),
                "x-wendao-repo-search-limit=5".to_owned(),
                format!("x-wendao-repo-search-path-prefixes={}", section.source_path),
            ],
            "note": "Resolve the Markdown section candidate to a page hit through native repo search.",
            "requiresResolvedPageId": false,
            "requiresResolvedNodeId": false,
        }),
        json!({
            "step": "flight_resolve_page_index_tree",
            "transport": "arrow-flight",
            "route": "/analysis/repo-projected-page-index-tree",
            "metadataTemplates": page_index_metadata,
            "note": "Select the concrete page-index node from the returned tree; do not treat the Markdown anchor as the node id.",
            "requiresResolvedPageId": true,
            "requiresResolvedNodeId": false,
        }),
        json!({
            "step": "flight_open_retrieval_context",
            "transport": "arrow-flight",
            "route": "/analysis/repo-projected-retrieval-context",
            "metadataTemplates": [
                "x-wendao-repo-projected-retrieval-context-repo=<repo>",
                "x-wendao-repo-projected-retrieval-context-page-id=<resolved-page-id>",
                "x-wendao-repo-projected-retrieval-context-node-id=<resolved-node-id>",
                "x-wendao-repo-projected-retrieval-context-related-limit=5",
            ],
            "note": "Open the section-level projected retrieval context through the native Flight route.",
            "requiresResolvedPageId": true,
            "requiresResolvedNodeId": true,
        }),
        json!({
            "step": "flight_expand_graph_context",
            "transport": "arrow-flight",
            "route": "/graph/neighbors",
            "metadataTemplates": [
                "x-wendao-graph-node-id=<resolved-graph-node-id>",
                "x-wendao-graph-direction=both",
                "x-wendao-graph-hops=2",
                "x-wendao-graph-limit=50",
            ],
            "note": "Expand document-level graph context through the graph relation layer before the next reasoning-tree branch.",
            "requiresResolvedPageId": true,
            "requiresResolvedNodeId": true,
            "requiresResolvedGraphNodeId": true,
        }),
    ]
}

fn json_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn json_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// Runs the local `WendaoGraph.jl` `PageIndex` host-request probe in a real
/// Julia process.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or host fixture
/// cannot be resolved, the Julia process exits unsuccessfully, or the probe
/// output cannot be parsed.
pub fn probe_wendaograph_page_index_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = wendaograph_page_index_host_fixture_dir()?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        false,
        "PageIndex",
    )?;
    parse_page_index_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `PageIndex` host-request probe with an
/// explicit fixture directory.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or supplied host
/// fixture cannot be resolved, the Julia process exits unsuccessfully, or the
/// probe output cannot be parsed.
pub fn probe_wendaograph_page_index_host_request_with_fixture(
    fixture_dir: impl Into<PathBuf>,
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = resolve_existing_path("WendaoGraph PageIndex host fixture", fixture_dir)?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        false,
        "PageIndex",
    )?;
    parse_page_index_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `PageIndex` planner-action host probe in a
/// real Julia process.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or host fixture
/// cannot be resolved, the Julia process exits unsuccessfully, or the probe
/// output cannot be parsed.
pub fn probe_wendaograph_page_index_planner_action_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = wendaograph_page_index_host_fixture_dir()?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        true,
        "PageIndex planner-action",
    )?;
    parse_page_index_planner_action_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `PageIndex` planner-action host probe with
/// an explicit fixture directory.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project or supplied host
/// fixture cannot be resolved, the Julia process exits unsuccessfully, or the
/// probe output cannot be parsed.
pub fn probe_wendaograph_page_index_planner_action_host_request_with_fixture(
    fixture_dir: impl Into<PathBuf>,
    warm_sample_count: usize,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let fixture_dir = resolve_existing_path("WendaoGraph PageIndex host fixture", fixture_dir)?;
    let stdout = run_wendaograph_page_index_host_probe(
        &julia_project,
        &fixture_dir,
        warm_sample_count,
        true,
        "PageIndex planner-action",
    )?;
    parse_page_index_planner_action_probe_stdout(&stdout)
}

/// Runs the local `WendaoGraph.jl` `LinkGraph` host-request probe in a real
/// Julia process.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project cannot be resolved,
/// the Julia process exits unsuccessfully, or the probe output cannot be parsed.
pub fn probe_wendaograph_link_graph_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphLinkGraphHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let output = Command::new("julia")
        .arg(format!("--project={}", julia_project.display()))
        .arg("-e")
        .arg(LINK_GRAPH_HOST_PROBE_JULIA)
        .envs(link_graph_synthetic_envs())
        .env(
            WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV,
            warm_sample_count.max(1).to_string(),
        )
        .output()
        .map_err(|error| format!("spawn WendaoGraph LinkGraph host probe: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph LinkGraph host probe exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_link_graph_probe_stdout(stdout.as_ref())
}

/// Runs the local `WendaoGraph.jl` `LinkGraph` host-request probe and parses
/// the full structural row-count surface.
///
/// # Errors
///
/// Returns an error when the local `WendaoGraph.jl` project cannot be resolved,
/// the Julia process exits unsuccessfully, or the probe output cannot be parsed.
pub fn probe_wendaograph_link_graph_full_structural_host_request(
    warm_sample_count: usize,
) -> Result<WendaoGraphLinkGraphFullStructuralHostProbeReport, String> {
    let julia_project = wendaograph_julia_project()?;
    let output = Command::new("julia")
        .arg(format!("--project={}", julia_project.display()))
        .arg("-e")
        .arg(LINK_GRAPH_HOST_PROBE_JULIA)
        .envs(link_graph_synthetic_envs())
        .env(
            WENDAO_GRAPH_LINK_GRAPH_HOST_PROBE_WARM_SAMPLES_ENV,
            warm_sample_count.max(1).to_string(),
        )
        .output()
        .map_err(|error| {
            format!("spawn WendaoGraph LinkGraph full structural host probe: {error}")
        })?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph LinkGraph full structural host probe exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_link_graph_full_structural_probe_stdout(stdout.as_ref())
}

fn wendaograph_julia_project() -> Result<PathBuf, String> {
    if let Some(configured) = env::var_os(WENDAOGRAPH_JULIA_PROJECT_ENV) {
        return resolve_existing_path("WendaoGraph Julia project", configured);
    }
    if let Some(configured) = env::var_os(WENDAOGRAPH_PACKAGE_DIR_ENV) {
        return resolve_existing_path("WendaoGraph package dir", configured);
    }

    let candidate = repo_root().join(".data/WendaoGraph.jl");
    if candidate.is_dir() {
        return candidate.canonicalize().map_err(|error| {
            format!(
                "resolve default WendaoGraph package dir `{}`: {error}",
                candidate.display()
            )
        });
    }

    Err(format!(
        "WendaoGraph package dir not found at `{}`; set {WENDAOGRAPH_PACKAGE_DIR_ENV} or {WENDAOGRAPH_JULIA_PROJECT_ENV}",
        candidate.display()
    ))
}

fn wendaograph_page_index_host_fixture_dir() -> Result<PathBuf, String> {
    if let Some(configured) = env::var_os(WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV) {
        return resolve_existing_path("WendaoGraph PageIndex host fixture", configured);
    }
    resolve_existing_path(
        "WendaoGraph PageIndex host fixture",
        repo_root().join(
            "packages/rust/crates/xiuxian-wendao/tests/fixtures/wendaograph_page_index_reasoning_host",
        ),
    )
}

fn run_wendaograph_page_index_host_probe(
    julia_project: &Path,
    fixture_dir: &Path,
    warm_sample_count: usize,
    planner_actions: bool,
    label: &str,
) -> Result<String, String> {
    let mut command = Command::new("julia");
    command
        .arg(format!("--project={}", julia_project.display()))
        .arg("-e")
        .arg(PAGE_INDEX_HOST_PROBE_JULIA)
        .env(WENDAO_GRAPH_PAGE_INDEX_HOST_FIXTURE_ENV, fixture_dir)
        .env(
            WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_WARM_SAMPLES_ENV,
            warm_sample_count.max(1).to_string(),
        );
    if planner_actions {
        command.env(WENDAO_GRAPH_PAGE_INDEX_HOST_PROBE_ACTIONS_ENV, "1");
    }

    let output = command
        .output()
        .map_err(|error| format!("spawn WendaoGraph {label} host probe: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "WendaoGraph {label} host probe exited with status {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn link_graph_synthetic_envs() -> Vec<(&'static str, String)> {
    [
        WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_NODES_ENV,
        WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_FANOUT_ENV,
        WENDAO_GRAPH_LINK_GRAPH_SYNTHETIC_SEMANTIC_NEIGHBORS_ENV,
    ]
    .into_iter()
    .filter_map(|key| env::var(key).ok().map(|value| (key, value)))
    .collect()
}

fn resolve_existing_path(label: &str, configured: impl Into<PathBuf>) -> Result<PathBuf, String> {
    let candidate = configured.into();
    let candidate = if candidate.is_absolute() {
        candidate
    } else {
        repo_root().join(candidate)
    };
    candidate
        .canonicalize()
        .map_err(|error| format!("resolve {label} `{}`: {error}", candidate.display()))
}

fn parse_page_index_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let line = probe_report_line(stdout, PAGE_INDEX_HOST_PROBE_PREFIX, "PageIndex")?;
    parse_page_index_probe_report_line(line)
}

fn parse_page_index_planner_action_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let line = probe_report_line(
        stdout,
        PAGE_INDEX_HOST_PROBE_PREFIX,
        "PageIndex planner action",
    )?;
    parse_page_index_planner_action_probe_report_line(line)
}

fn parse_link_graph_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphLinkGraphHostProbeReport, String> {
    let line = probe_report_line(stdout, LINK_GRAPH_HOST_PROBE_PREFIX, "LinkGraph")?;
    parse_link_graph_probe_report_line(line)
}

fn parse_link_graph_full_structural_probe_stdout(
    stdout: &str,
) -> Result<WendaoGraphLinkGraphFullStructuralHostProbeReport, String> {
    let line = probe_report_line(
        stdout,
        LINK_GRAPH_HOST_PROBE_PREFIX,
        "LinkGraph full structural",
    )?;
    parse_link_graph_full_structural_probe_report_line(line)
}

fn probe_report_line<'a>(stdout: &'a str, prefix: &str, label: &str) -> Result<&'a str, String> {
    stdout
        .lines()
        .find(|line| line.starts_with(prefix))
        .ok_or_else(|| {
            format!("WendaoGraph {label} host probe did not print `{prefix}`; stdout:\n{stdout}")
        })
}

fn parse_page_index_probe_report_line(
    line: &str,
) -> Result<WendaoGraphPageIndexHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphPageIndexHostProbeReport {
        sample_count: parse_usize_field(&fields, "sample_count")?,
        first_ms: parse_f64_field(&fields, "first_ms")?,
        warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
        warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
        warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
        warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
        frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
        trace_rows: parse_usize_field(&fields, "trace_rows")?,
    })
}

fn parse_page_index_planner_action_probe_report_line(
    line: &str,
) -> Result<WendaoGraphPageIndexPlannerActionHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphPageIndexPlannerActionHostProbeReport {
        base: WendaoGraphPageIndexHostProbeReport {
            sample_count: parse_usize_field(&fields, "sample_count")?,
            first_ms: parse_f64_field(&fields, "first_ms")?,
            warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
            warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
            warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
            warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
            frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
            trace_rows: parse_usize_field(&fields, "trace_rows")?,
        },
        planner_action_rows: parse_usize_field(&fields, "planner_action_rows")?,
        planner_expand_actions: parse_usize_field(&fields, "planner_expand_actions")?,
        planner_compare_actions: parse_usize_field(&fields, "planner_compare_actions")?,
        planner_jump_actions: parse_usize_field(&fields, "planner_jump_actions")?,
        planner_stop_actions: parse_usize_field(&fields, "planner_stop_actions")?,
    })
}

fn parse_link_graph_probe_report_line(
    line: &str,
) -> Result<WendaoGraphLinkGraphHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphLinkGraphHostProbeReport {
        mode: parse_string_field_or(&fields, "mode", "semantic-neighbors").to_owned(),
        node_count: parse_usize_field_or(&fields, "node_count", 4)?,
        edge_count: parse_usize_field_or(&fields, "edge_count", 2)?,
        semantic_neighbor_count: parse_usize_field_or(&fields, "semantic_neighbor_count", 1)?,
        sample_count: parse_usize_field(&fields, "sample_count")?,
        first_ms: parse_f64_field(&fields, "first_ms")?,
        warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
        warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
        warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
        warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
        graph_metric_rows: parse_usize_field(&fields, "graph_metric_rows")?,
        topology_candidate_rows: parse_usize_field(&fields, "topology_candidate_rows")?,
        semantic_overlay_rows: parse_usize_field(&fields, "semantic_overlay_rows")?,
        diffusion_rows: parse_usize_field(&fields, "diffusion_rows")?,
        frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
    })
}

fn parse_link_graph_full_structural_probe_report_line(
    line: &str,
) -> Result<WendaoGraphLinkGraphFullStructuralHostProbeReport, String> {
    let fields = parse_probe_fields(line)?;

    Ok(WendaoGraphLinkGraphFullStructuralHostProbeReport {
        base: WendaoGraphLinkGraphHostProbeReport {
            mode: parse_string_field_or(&fields, "mode", "semantic-neighbors").to_owned(),
            node_count: parse_usize_field_or(&fields, "node_count", 4)?,
            edge_count: parse_usize_field_or(&fields, "edge_count", 2)?,
            semantic_neighbor_count: parse_usize_field_or(&fields, "semantic_neighbor_count", 1)?,
            sample_count: parse_usize_field(&fields, "sample_count")?,
            first_ms: parse_f64_field(&fields, "first_ms")?,
            warm_min_ms: parse_f64_field(&fields, "warm_min_ms")?,
            warm_median_ms: parse_f64_field(&fields, "warm_median_ms")?,
            warm_p95_ms: parse_f64_field(&fields, "warm_p95_ms")?,
            warm_max_ms: parse_f64_field(&fields, "warm_max_ms")?,
            graph_metric_rows: parse_usize_field(&fields, "graph_metric_rows")?,
            topology_candidate_rows: parse_usize_field(&fields, "topology_candidate_rows")?,
            semantic_overlay_rows: parse_usize_field(&fields, "semantic_overlay_rows")?,
            diffusion_rows: parse_usize_field(&fields, "diffusion_rows")?,
            frontier_rows: parse_usize_field(&fields, "frontier_rows")?,
        },
        component_rows: parse_usize_field(&fields, "component_rows")?,
        topology_profile_rows: parse_usize_field(&fields, "topology_profile_rows")?,
        topology_bottleneck_rows: parse_usize_field(&fields, "topology_bottleneck_rows")?,
        topology_community_rows: parse_usize_field(&fields, "topology_community_rows")?,
        topology_cover_rows: parse_usize_field(&fields, "topology_cover_rows")?,
        topology_core_rows: parse_usize_field(&fields, "topology_core_rows")?,
        topology_boundary_rows: parse_usize_field(&fields, "topology_boundary_rows")?,
        topology_transition_rows: parse_usize_field(&fields, "topology_transition_rows")?,
        topology_gateway_rows: parse_usize_field(&fields, "topology_gateway_rows")?,
        topology_community_summary_rows: parse_usize_field(
            &fields,
            "topology_community_summary_rows",
        )?,
        topology_community_link_rows: parse_usize_field(&fields, "topology_community_link_rows")?,
        topology_community_frontier_rows: parse_usize_field(
            &fields,
            "topology_community_frontier_rows",
        )?,
    })
}

fn parse_probe_fields(line: &str) -> Result<HashMap<&str, &str>, String> {
    let mut fields = HashMap::new();
    for token in line.split_whitespace().skip(1) {
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| format!("invalid probe token `{token}`"))?;
        fields.insert(key, value);
    }
    Ok(fields)
}

fn parse_usize_field(fields: &HashMap<&str, &str>, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing probe field `{key}`"))?
        .parse()
        .map_err(|error| format!("parse probe field `{key}` as usize: {error}"))
}

fn parse_usize_field_or(
    fields: &HashMap<&str, &str>,
    key: &str,
    default_value: usize,
) -> Result<usize, String> {
    fields.get(key).map_or(Ok(default_value), |value| {
        value
            .parse()
            .map_err(|error| format!("parse probe field `{key}` as usize: {error}"))
    })
}

fn parse_string_field_or<'a>(
    fields: &'a HashMap<&str, &str>,
    key: &str,
    default_value: &'a str,
) -> &'a str {
    fields.get(key).copied().unwrap_or(default_value)
}

fn parse_f64_field(fields: &HashMap<&str, &str>, key: &str) -> Result<f64, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing probe field `{key}`"))?
        .parse()
        .map_err(|error| format!("parse probe field `{key}` as f64: {error}"))
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaograph.rs"]
mod tests;
