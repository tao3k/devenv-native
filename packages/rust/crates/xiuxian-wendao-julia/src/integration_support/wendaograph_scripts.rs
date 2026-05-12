//! Embedded Julia scripts for `WendaoGraph` integration probes.

pub(crate) const SEARCH_STRATEGY_FLOW_JULIA: &str = r#"
using WendaoGraph

batch_mode = !isempty(ARGS) && ARGS[1] == "__WENDAO_SEARCH_STRATEGY_FLOW_BATCH__"
persistent_batch_mode = !isempty(ARGS) && ARGS[1] == "__WENDAO_SEARCH_STRATEGY_FLOW_BATCH_STDIN__"
if batch_mode
    length(ARGS) >= 3 || error("SearchStrategyFlow batch mode requires search root and batch count")
    search_root = ARGS[2]
elseif persistent_batch_mode
    length(ARGS) >= 2 || error("SearchStrategyFlow persistent batch mode requires search root")
    search_root = ARGS[2]
else
    intent = ARGS[1]
    search_root = ARGS[2]
    candidate_input_tsv = length(ARGS) >= 3 ? ARGS[3] : ""
    candidate_input_source_hint = length(ARGS) >= 4 ? ARGS[4] : ""
    candidate_input_discovery_json = length(ARGS) >= 5 ? ARGS[5] : "null"
end

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

function json_raw_or_null(value)
    stripped = strip(String(value))
    isempty(stripped) && return "null"
    (startswith(stripped, "{") || startswith(stripped, "[")) && return stripped
    stripped == "null" && return "null"
    "null"
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

function fixed_proof_candidates(strategy_weight, page_index_weight)
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

function search_strategy_flow_json(intent, candidate_input_tsv, candidate_input_source_hint, candidate_input_discovery_json)
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

candidate_inputs = parse_candidate_inputs(candidate_input_tsv)
candidate_input_source = isempty(candidate_inputs) ? "fixed-proof-fallback" : (isempty(candidate_input_source_hint) ? "rust-markdown-headings" : candidate_input_source_hint)
candidate_input_discovery = json_raw_or_null(candidate_input_discovery_json)
candidates = isempty(candidate_inputs) ? fixed_proof_candidates(strategy_weight, page_index_weight) : doc_candidate_from_input.(candidate_inputs)

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
    query_understanding = query_understanding,
)
required_evidence_coverage = strategy_flow_required_evidence_coverage(frontier, query_understanding)
actions = strategy_flow_planner_action_rows(
    rows,
    transitions,
    frontier;
    flow_id = flow_id,
    loop_budget = strategy_budget.loop_budget,
    judgement_budget = strategy_budget.judgement_budget,
    compare_count = 1,
)

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
    "requiredEvidenceCovered" => required_evidence_coverage.required_evidence_covered,
    "selectedRequiredEvidence" => required_evidence_coverage.selected_required_evidence,
    "missingRequiredEvidence" => required_evidence_coverage.missing_required_evidence,
))

return "{" * join((
    json_pair("intent", json_value(intent)),
    json_pair("backend", json_value("rust-wendao-julia")),
    json_pair("controlPlane", json_value("rust")),
    json_pair("juliaProject", json_value(Base.active_project() === nothing ? "" : dirname(Base.active_project()))),
    json_pair("graphProject", json_value(Base.active_project() === nothing ? "" : dirname(Base.active_project()))),
    json_pair("searchRoot", json_value(search_root)),
    json_pair("candidateInputSource", json_value(candidate_input_source)),
    json_pair("candidateInputCount", json_value(length(candidate_inputs))),
    json_pair("candidateInputDiscovery", candidate_input_discovery),
    json_pair("queryUnderstanding", "[" * join(query_understanding_json.(query_understanding), ",") * "]"),
    json_pair("strategyBudget", strategy_budget_json(strategy_budget)),
    json_pair("stageReceipts", "[" * join(stage_receipt_json.(stage_receipts), ",") * "]"),
    json_pair("candidates", "[" * join(candidate_json.(rows), ",") * "]"),
    json_pair("frontier", "[" * join(frontier_json.(frontier), ",") * "]"),
    json_pair("plannerActions", "[" * join(action_json.(actions), ",") * "]"),
    json_pair("summary", summary),
    json_pair("validation", validation),
), ",") * "}"
end

if batch_mode
    batch_count = parse(Int, ARGS[3])
    expected_arg_count = 3 + batch_count * 4
    length(ARGS) == expected_arg_count || error("SearchStrategyFlow batch mode expected $expected_arg_count args, got $(length(ARGS))")

    function search_strategy_flow_batch_json_lines(batch_count)
        traces = String[]
        arg_index = 4
        for _ in 1:batch_count
            batch_intent = ARGS[arg_index]
            batch_candidate_input_tsv = ARGS[arg_index + 1]
            batch_candidate_input_source_hint = ARGS[arg_index + 2]
            batch_candidate_input_discovery_json = ARGS[arg_index + 3]
            push!(traces, search_strategy_flow_json(batch_intent, batch_candidate_input_tsv, batch_candidate_input_source_hint, batch_candidate_input_discovery_json))
            arg_index += 4
        end
        join(traces, "\n")
    end

    println(search_strategy_flow_batch_json_lines(batch_count))
elseif persistent_batch_mode
    function read_search_strategy_flow_payload()
        length_line = try
            readline(stdin)
        catch error
            error isa EOFError && return nothing
            rethrow(error)
        end
        isempty(length_line) && eof(stdin) && return nothing
        byte_count = parse(Int, length_line)
        String(read(stdin, byte_count))
    end

    while true
        batch_count_payload = read_search_strategy_flow_payload()
        batch_count_payload === nothing && break
        local batch_count = parse(Int, batch_count_payload)
        local request_batches = NamedTuple[]
        for _ in 1:batch_count
            batch_intent = read_search_strategy_flow_payload()
            batch_candidate_input_tsv = read_search_strategy_flow_payload()
            batch_candidate_input_source_hint = read_search_strategy_flow_payload()
            batch_candidate_input_discovery_json = read_search_strategy_flow_payload()
            push!(
                request_batches,
                (
                    intent = batch_intent,
                    candidate_input_tsv = batch_candidate_input_tsv,
                    candidate_input_source_hint = batch_candidate_input_source_hint,
                    candidate_input_discovery_json = batch_candidate_input_discovery_json,
                ),
            )
        end
        traces = [
            search_strategy_flow_json(batch.intent, batch.candidate_input_tsv, batch.candidate_input_source_hint, batch.candidate_input_discovery_json)
            for batch in request_batches
        ]
        println(join(traces, "\n"))
        flush(stdout)
    end
else
    println(search_strategy_flow_json(intent, candidate_input_tsv, candidate_input_source_hint, candidate_input_discovery_json))
end
"#;

pub(crate) const PAGE_INDEX_HOST_PROBE_JULIA: &str = r#"
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

pub(crate) const LINK_GRAPH_HOST_PROBE_JULIA: &str = r#"
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
