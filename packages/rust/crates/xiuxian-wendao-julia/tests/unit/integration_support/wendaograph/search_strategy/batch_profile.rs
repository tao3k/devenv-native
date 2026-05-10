use std::{env, path::Path, time::Instant};

use super::{
    SearchStrategyFlowLiveReplayRunReport, assert_configured_markdown_batch_replay_traces,
    assert_configured_markdown_live_replay_reports, configured_markdown_replay_inputs,
    print_configured_markdown_live_replay_reports, run_configured_markdown_live_replay_reports,
    run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches,
    search_strategy_flow_live_replay_search_root,
};

const RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_CANDIDATE_BUDGET_PROFILE_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_CANDIDATE_BUDGET_PROFILE_TEST";
const RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_REPLAY_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_REPLAY_TEST";
const RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_CANDIDATE_BUDGET_PROFILE_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_CANDIDATE_BUDGET_PROFILE_TEST";

#[derive(Debug, Clone, Copy)]
struct SearchStrategyFlowCandidateBudgetProfileReport {
    max_candidates_per_family: usize,
    family_count: usize,
    surface_markdown_file_count: usize,
    surface_heading_count: usize,
    input_candidate_count: usize,
    selected_frontier_count: usize,
    planner_action_count: usize,
    route_count: usize,
    projected_row_count: usize,
    elapsed_ms: u128,
}

#[derive(Debug, Clone)]
struct SearchStrategyFlowBatchCandidateBudgetProfileReport {
    profiles: Vec<SearchStrategyFlowCandidateBudgetProfileReport>,
    julia_process_count: usize,
    elapsed_ms: u128,
}

#[test]
fn wendaograph_search_strategy_flow_profiles_candidate_budgets_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_CANDIDATE_BUDGET_PROFILE_TEST_ENV).is_none()
    {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow candidate budget profile; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_CANDIDATE_BUDGET_PROFILE_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    let profiles = [12, 24, 48]
        .into_iter()
        .map(|limit| {
            candidate_budget_profile_report(
                limit,
                &run_configured_markdown_live_replay_reports(search_root.as_path(), Some(limit)),
            )
        })
        .collect::<Vec<_>>();
    assert_candidate_budget_profile_reports(&profiles);
}

#[test]
fn wendaograph_search_strategy_flow_batch_replay_runs_configured_markdown_families_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_REPLAY_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow batch replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_REPLAY_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    let report = run_configured_markdown_batch_replay_reports(search_root.as_path(), None);
    assert_configured_markdown_live_replay_reports(&report.family_reports);
    eprintln!(
        "SearchStrategyFlow batch replay summary: families={}, juliaProcesses=1, elapsedMs={}",
        report.family_reports.len(),
        report.elapsed_ms
    );
}

#[test]
fn wendaograph_search_strategy_flow_batch_profiles_candidate_budgets_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_CANDIDATE_BUDGET_PROFILE_TEST_ENV)
        .is_none()
    {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow batch candidate budget profile; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BATCH_CANDIDATE_BUDGET_PROFILE_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    let report = run_configured_markdown_batch_candidate_budget_profile_reports(
        search_root.as_path(),
        &[12, 24, 48],
    );
    assert_candidate_budget_profile_reports(&report.profiles);
    eprintln!(
        "SearchStrategyFlow batch candidate budget profile summary: budgets={}, juliaProcesses={}, elapsedMs={}",
        report.profiles.len(),
        report.julia_process_count,
        report.elapsed_ms
    );
}

fn candidate_budget_profile_report(
    max_candidates_per_family: usize,
    report: &SearchStrategyFlowLiveReplayRunReport,
) -> SearchStrategyFlowCandidateBudgetProfileReport {
    let reports = &report.family_reports;
    SearchStrategyFlowCandidateBudgetProfileReport {
        max_candidates_per_family,
        family_count: reports.len(),
        surface_markdown_file_count: reports
            .iter()
            .map(|report| report.surface_markdown_file_count)
            .sum(),
        surface_heading_count: reports
            .iter()
            .map(|report| report.surface_heading_count)
            .sum(),
        input_candidate_count: reports
            .iter()
            .map(|report| report.input_candidate_count)
            .sum(),
        selected_frontier_count: reports
            .iter()
            .map(|report| report.selected_frontier_count)
            .sum(),
        planner_action_count: reports
            .iter()
            .map(|report| report.planner_action_count)
            .sum(),
        route_count: reports.iter().map(|report| report.route_count).sum(),
        projected_row_count: reports
            .iter()
            .map(|report| report.projected_row_count)
            .sum(),
        elapsed_ms: report.elapsed_ms,
    }
}

fn assert_candidate_budget_profile_reports(
    profiles: &[SearchStrategyFlowCandidateBudgetProfileReport],
) {
    assert_eq!(
        profiles.len(),
        3,
        "candidate budget profile should cover the configured budget ladder"
    );
    assert_eq!(
        profiles
            .iter()
            .map(|profile| profile.max_candidates_per_family)
            .collect::<Vec<_>>(),
        vec![12, 24, 48],
        "candidate budget profile should keep the documented ladder"
    );

    let mut previous_input_count = 0;
    for profile in profiles {
        eprintln!(
            "SearchStrategyFlow candidate budget profile: maxCandidatesPerFamily={}, families={}, surfaceMarkdownFiles={}, surfaceHeadings={}, inputCandidates={}, selectedFrontier={}, plannerActions={}, routes={}, projectedRows={}, elapsedMs={}",
            profile.max_candidates_per_family,
            profile.family_count,
            profile.surface_markdown_file_count,
            profile.surface_heading_count,
            profile.input_candidate_count,
            profile.selected_frontier_count,
            profile.planner_action_count,
            profile.route_count,
            profile.projected_row_count,
            profile.elapsed_ms
        );
        assert_eq!(
            profile.family_count, 4,
            "candidate budget profile should preserve the four configured replay families"
        );
        assert!(
            profile.surface_markdown_file_count >= 400,
            "candidate budget profile must cover the configured Markdown surface"
        );
        assert!(
            profile.surface_heading_count >= profile.surface_markdown_file_count,
            "candidate budget profile must preserve heading evidence"
        );
        assert!(
            profile.input_candidate_count >= previous_input_count,
            "candidate budget profile input rows must scale monotonically"
        );
        assert!(
            profile.input_candidate_count
                <= profile.max_candidates_per_family * profile.family_count,
            "candidate budget profile must respect the per-family cap"
        );
        assert!(
            profile.selected_frontier_count >= profile.family_count,
            "candidate budget profile must keep selected frontier rows"
        );
        assert!(
            profile.planner_action_count >= profile.family_count * 2,
            "candidate budget profile must keep non-trivial planner actions"
        );
        assert!(
            profile.route_count >= profile.family_count,
            "candidate budget profile must keep planned route receipts"
        );
        assert_eq!(
            profile.projected_row_count, profile.input_candidate_count,
            "candidate budget profile must project one Rust evidence row per input candidate"
        );
        assert!(
            profile.elapsed_ms > 0,
            "candidate budget profile must report measured wall span"
        );
        previous_input_count = profile.input_candidate_count;
    }
}

fn run_configured_markdown_batch_replay_reports(
    search_root: &Path,
    max_candidates_per_family: Option<usize>,
) -> SearchStrategyFlowLiveReplayRunReport {
    let replay_inputs = configured_markdown_replay_inputs(search_root, max_candidates_per_family);
    let candidate_batches = replay_inputs
        .iter()
        .map(|input| (input.spec.intent, input.batch.clone()))
        .collect::<Vec<_>>();

    let started = Instant::now();
    let traces = run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches(
        search_root,
        candidate_batches,
    )
    .unwrap_or_else(|error| panic!("run batch SearchStrategyFlow replay: {error}"));
    let elapsed_ms = started.elapsed().as_millis();
    let reports = assert_configured_markdown_batch_replay_traces(&replay_inputs, &traces);
    print_configured_markdown_live_replay_reports(max_candidates_per_family, &reports, elapsed_ms);
    SearchStrategyFlowLiveReplayRunReport {
        family_reports: reports,
        elapsed_ms,
    }
}

fn run_configured_markdown_batch_candidate_budget_profile_reports(
    search_root: &Path,
    budget_limits: &[usize],
) -> SearchStrategyFlowBatchCandidateBudgetProfileReport {
    let replay_inputs_by_budget = budget_limits
        .iter()
        .flat_map(|limit| {
            configured_markdown_replay_inputs(search_root, Some(*limit))
                .into_iter()
                .map(|input| (*limit, input))
        })
        .collect::<Vec<_>>();
    let candidate_batches = replay_inputs_by_budget
        .iter()
        .map(|(_, input)| (input.spec.intent, input.batch.clone()))
        .collect::<Vec<_>>();

    let started = Instant::now();
    let traces = run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches(
        search_root,
        candidate_batches,
    )
    .unwrap_or_else(|error| {
        panic!("run batch SearchStrategyFlow candidate budget profile: {error}")
    });
    let elapsed_ms = started.elapsed().as_millis();
    assert_eq!(
        traces.len(),
        replay_inputs_by_budget.len(),
        "batch candidate budget profile must return one trace per replay input"
    );

    let profiles = budget_limits
        .iter()
        .map(|limit| {
            let budget_inputs = replay_inputs_by_budget
                .iter()
                .filter_map(|(input_limit, input)| {
                    (*input_limit == *limit).then_some(input.clone())
                })
                .collect::<Vec<_>>();
            let budget_traces = replay_inputs_by_budget
                .iter()
                .zip(traces.iter())
                .filter_map(|((input_limit, _), trace)| {
                    (*input_limit == *limit).then_some(trace.clone())
                })
                .collect::<Vec<_>>();
            let family_reports =
                assert_configured_markdown_batch_replay_traces(&budget_inputs, &budget_traces);
            print_configured_markdown_live_replay_reports(
                Some(*limit),
                &family_reports,
                elapsed_ms,
            );
            let report = SearchStrategyFlowLiveReplayRunReport {
                family_reports,
                elapsed_ms,
            };
            candidate_budget_profile_report(*limit, &report)
        })
        .collect::<Vec<_>>();

    SearchStrategyFlowBatchCandidateBudgetProfileReport {
        profiles,
        julia_process_count: 1,
        elapsed_ms,
    }
}
