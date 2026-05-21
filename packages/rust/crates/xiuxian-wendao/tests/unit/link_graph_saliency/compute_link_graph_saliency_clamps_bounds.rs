use super::support::assert_snapshot_eq;
use serde::Serialize;
use xiuxian_wendao::link_graph::{LinkGraphSaliencyPolicy, compute_link_graph_saliency};

#[derive(Serialize)]
struct ClampBoundsSnapshot {
    decayed: String,
    boosted: String,
}

#[test]
fn test_compute_link_graph_saliency_clamps_bounds() -> Result<(), String> {
    let policy = LinkGraphSaliencyPolicy {
        alpha: 0.5,
        minimum: 1.0,
        maximum: 10.0,
    };

    let decayed = compute_link_graph_saliency(5.0, 0.10, 0, 30.0, policy);
    let boosted = compute_link_graph_saliency(5.0, 0.0, 10_000, 0.0, policy);

    let snapshot = ClampBoundsSnapshot {
        decayed: format!("{decayed:.6}"),
        boosted: format!("{boosted:.6}"),
    };
    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(&snapshot).map_err(|err| err.to_string())?
    );
    assert_snapshot_eq(
        "link_graph/saliency/compute_link_graph_saliency_clamps_bounds.json",
        actual.as_str(),
    );
    Ok(())
}
