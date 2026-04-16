use super::support::*;
use std::collections::HashSet;

#[test]
fn test_agentic_expansion_plan_respects_worker_and_pair_budgets() -> TestResult {
    let fixture = build_index_fixture(&[
        ("notes/a.md", "---\ntags:\n  - alpha\n---\n# A\n\ncontent\n"),
        ("notes/b.md", "---\ntags:\n  - alpha\n---\n# B\n\ncontent\n"),
        ("notes/c.md", "---\ntags:\n  - beta\n---\n# C\n\ncontent\n"),
        ("notes/d.md", "---\ntags:\n  - gamma\n---\n# D\n\ncontent\n"),
    ])?;
    let index = &fixture.index;
    let plan = index.agentic_expansion_plan_with_config(None, expansion_config(2, 4, 2));

    assert_eq!(plan.total_notes, 4);
    assert_eq!(plan.candidate_notes, 4);
    assert_eq!(plan.total_possible_pairs, 6);
    assert!(plan.workers.len() <= 2);
    assert!(plan.workers.iter().all(|worker| worker.pair_count <= 2));
    assert!(plan.selected_pairs <= 4);
    assert_eq!(
        plan.selected_pairs,
        plan.workers
            .iter()
            .map(|worker| worker.pair_count)
            .sum::<usize>()
    );

    let mut seen_pairs = HashSet::<(String, String)>::new();
    for worker in &plan.workers {
        for pair in &worker.pairs {
            let key = if pair.left_id <= pair.right_id {
                (pair.left_id.clone(), pair.right_id.clone())
            } else {
                (pair.right_id.clone(), pair.left_id.clone())
            };
            assert!(seen_pairs.insert(key), "duplicate candidate pair in plan");
        }
    }

    Ok(())
}

#[test]
fn test_agentic_expansion_plan_query_narrows_candidates() -> TestResult {
    let fixture = build_index_fixture(&[
        ("docs/a.md", "# A\n\nalpha momentum\n"),
        ("docs/b.md", "# B\n\nalpha breakout\n"),
        ("docs/c.md", "# C\n\nbeta mean reversion\n"),
        ("docs/d.md", "# D\n\ngamma divergence\n"),
    ])?;
    let index = &fixture.index;
    let plan = index.agentic_expansion_plan_with_config(Some("alpha"), expansion_config(3, 10, 3));

    assert_eq!(plan.query.as_deref(), Some("alpha"));
    assert!(plan.candidate_notes <= 2);
    assert!(plan.selected_pairs <= 1);
    assert!(plan.workers.len() <= 1);

    Ok(())
}
