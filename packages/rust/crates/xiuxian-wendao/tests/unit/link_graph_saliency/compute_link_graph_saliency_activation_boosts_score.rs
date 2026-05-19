use super::support::assert_snapshot_eq;
use serde::Serialize;
use xiuxian_wendao::link_graph::{LinkGraphSaliencyPolicy, compute_link_graph_saliency};

#[derive(Serialize)]
struct ActivationBoostSnapshot {
    without_activation: String,
    with_activation: String,
}

#[test]
fn test_compute_link_graph_saliency_activation_boosts_score() -> Result<(), String> {
    let policy = LinkGraphSaliencyPolicy::default();
    let without_activation = compute_link_graph_saliency(5.0, 0.02, 0, 2.0, policy);
    let with_activation = compute_link_graph_saliency(5.0, 0.02, 8, 2.0, policy);

    let snapshot = ActivationBoostSnapshot {
        without_activation: format!("{without_activation:.6}"),
        with_activation: format!("{with_activation:.6}"),
    };
    let actual = format!(
        "{}\n",
        serde_json::to_string_pretty(&snapshot).map_err(|err| err.to_string())?
    );
    assert_snapshot_eq(
        "link_graph/saliency/compute_link_graph_saliency_activation_boosts_score.json",
        actual.as_str(),
    );
    Ok(())
}
