use std::{env, fs, path::PathBuf};

use super::materialized_search_strategy_flow_markdown_replay_families_from_bridge_report;

#[test]
fn materialized_repo_replay_families_consume_benchmark_ready_bridge_rows_only()
-> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let ready_checkout = temp_dir.path().join("ready-repo");
    let backlog_checkout = temp_dir.path().join("backlog-repo");
    fs::create_dir_all(ready_checkout.join("docs"))?;
    fs::write(
        ready_checkout.join("docs/search.md"),
        "# Search Strategy Flow\n\n## Graph Evidence\n\nSearchStrategyFlow LinkGraph PageIndex evidence.\n",
    )?;
    let report_path = temp_dir.path().join("bridge-report.json");
    fs::write(
        &report_path,
        format!(
            r#"{{
  "rows": [
    {{
      "repoId": "ReadyRepo.jl",
      "checkoutPath": "{}",
      "benchmarkEligible": true,
      "prewarmAction": "benchmark_ready"
    }},
    {{
      "repoId": "BacklogRepo.jl",
      "checkoutPath": "{}",
      "benchmarkEligible": false,
      "prewarmAction": "prewarm_required"
    }}
  ]
}}"#,
            ready_checkout.display(),
            backlog_checkout.display(),
        ),
    )?;

    let families = materialized_search_strategy_flow_markdown_replay_families_from_bridge_report(
        &report_path,
        "SearchStrategyFlow LinkGraph PageIndex",
        None,
        Some(8),
    )?;

    assert_eq!(families.len(), 1);
    let family = &families[0];
    assert_eq!(family.repo_id, "ReadyRepo.jl");
    assert_eq!(family.markdown_file_count, 1);
    assert_eq!(family.heading_count, 2);
    assert_eq!(
        family.batch.source,
        "rust-materialized-repo-markdown-headings"
    );
    assert!(family.batch.row_count > 0);
    assert!(
        family
            .batch
            .tsv
            .lines()
            .all(|line| line.starts_with("repos/ReadyRepo.jl/")),
        "materialized candidate paths should keep repo provenance"
    );
    assert!(!family.batch.tsv.contains("BacklogRepo.jl"));
    Ok(())
}

#[test]
fn real_bridge_report_materialized_repo_replay_when_enabled()
-> Result<(), Box<dyn std::error::Error>> {
    if env::var("RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_REPO_REPLAY_TEST").as_deref()
        != Ok("1")
    {
        eprintln!(
            "skipped: set RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_REPO_REPLAY_TEST=1"
        );
        return Ok(());
    }

    let report_path = env::var_os("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BRIDGE_AUDIT_REPORT")
        .map_or_else(
            || {
            PathBuf::from(env::var_os("PRJ_CACHE_HOME").unwrap_or_else(|| ".cache".into())).join(
                "agent/reports/2026-05-10-wendaograph-real-scenario-rust-bridge-resource-audit.json",
            )
            },
            PathBuf::from,
        );
    let intent = env::var("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_REPLAY_INTENT")
        .unwrap_or_else(|_| "SearchStrategyFlow PageIndex LinkGraph evidence strategy".to_owned());
    let max_repos = env::var("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_REPLAY_MAX_REPOS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(32);
    let max_candidates_per_repo =
        env::var("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_REPLAY_MAX_CANDIDATES_PER_REPO")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(8);

    let families = materialized_search_strategy_flow_markdown_replay_families_from_bridge_report(
        &report_path,
        &intent,
        Some(max_repos),
        Some(max_candidates_per_repo),
    )?;

    assert!(
        !families.is_empty(),
        "expected at least one materialized benchmark-ready replay family from {}",
        report_path.display()
    );
    let repo_count = families.len();
    let markdown_file_count: usize = families
        .iter()
        .map(|family| family.markdown_file_count)
        .sum();
    let heading_count: usize = families.iter().map(|family| family.heading_count).sum();
    let candidate_count: usize = families.iter().map(|family| family.batch.row_count).sum();
    for family in &families {
        assert!(family.checkout_path.is_dir());
        assert!(family.markdown_file_count > 0);
        assert!(family.heading_count > 0);
        assert!(family.batch.row_count > 0);
        assert_eq!(
            family.batch.source,
            "rust-materialized-repo-markdown-headings"
        );
        assert!(
            family
                .batch
                .tsv
                .lines()
                .all(|line| line.starts_with(&format!("repos/{}/", family.repo_id))),
            "materialized replay TSV should preserve repo provenance for {}",
            family.repo_id
        );
    }
    eprintln!(
        "materialized SearchStrategyFlow replay: report={}, requestedRepos={}, replayRepos={}, markdownFiles={}, headings={}, candidates={}",
        report_path.display(),
        max_repos,
        repo_count,
        markdown_file_count,
        heading_count,
        candidate_count
    );
    Ok(())
}
