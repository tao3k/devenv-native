pub(super) use super::super::*;

pub(super) fn gap_matches_needle(gap: &serde_json::Map<String, Value>, needle: &str) -> bool {
    let title = gap.get("title").and_then(Value::as_str).unwrap_or_default();
    let page_id = gap
        .get("page_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    title.contains(needle) || page_id.contains(needle)
}

pub(super) fn hit_gap_matches_needle(hit: &Value, needle: &str) -> bool {
    hit.get("gap")
        .and_then(Value::as_object)
        .is_some_and(|gap| gap_matches_needle(gap, needle))
}

pub(super) fn group_preview_within_limit(group: &Value, limit: usize) -> bool {
    group
        .get("gaps")
        .and_then(Value::as_array)
        .is_some_and(|gaps| gaps.len() <= limit)
}

pub(super) fn group_gaps_match_needle(group: &Value, needle: &str) -> bool {
    group
        .get("gaps")
        .and_then(Value::as_array)
        .is_some_and(|gaps| {
            gaps.iter().all(|gap| {
                gap.as_object()
                    .is_some_and(|gap| gap_matches_needle(gap, needle))
            })
        })
}

pub(super) fn sum_u64_field(values: &[Value], field: &str) -> u64 {
    values
        .iter()
        .map(|value| value.get(field).and_then(Value::as_u64).unwrap_or_default())
        .sum()
}

pub(super) fn selected_count_sum(values: &[Value]) -> Option<usize> {
    values.iter().try_fold(0usize, |acc, value| {
        let count = value.get("selected_count").and_then(Value::as_u64)?;
        let count = usize::try_from(count).ok()?;
        acc.checked_add(count)
    })
}

pub(super) fn planner_rank_key(hit: &Value) -> (std::cmp::Reverse<i64>, String, String, String) {
    let gap = hit.get("gap").and_then(Value::as_object);
    (
        std::cmp::Reverse(
            hit.get("priority_score")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
        ),
        gap.and_then(|gap| gap.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        gap.and_then(|gap| gap.get("title"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        gap.and_then(|gap| gap.get("gap_id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    )
}

pub(super) fn modelica_nodocs_router(
    repo_id: &str,
) -> Result<(tempfile::TempDir, axum::Router), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_local_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        repo_dir.join("Controllers").join("NoDocs.mo"),
        "within Projectionica.Controllers;\nmodel NoDocs\nend NoDocs;\n",
    )?;
    write_modelica_repo_config(temp.path(), &repo_dir, repo_id)?;
    let router = studio_router(gateway_state_for_project(temp.path()));
    Ok((temp, router))
}
