use xiuxian_qianji_control::HotStateSnapshot;

use super::{render_hot_state_snapshot_json, render_hot_state_snapshot_text};

#[test]
fn hot_state_text_renderer_reports_empty_snapshot_counts() {
    let snapshot = HotStateSnapshot::new(42);
    let rendered = render_hot_state_snapshot_text(&snapshot);

    assert!(rendered.contains("# Qianji Control Hot State"));
    assert!(rendered.contains("- Observed at ms: `42`"));
    assert!(rendered.contains("- Pending steps: `0`"));
    assert!(rendered.contains("- Active leases: `0`"));
    assert!(rendered.contains("- Live worker heartbeats: `0`"));
}

#[test]
fn hot_state_json_renderer_preserves_observation_time() {
    let snapshot = HotStateSnapshot::new(77);
    let rendered = match render_hot_state_snapshot_json(&snapshot) {
        Ok(rendered) => rendered,
        Err(error) => panic!("hot-state JSON render should succeed: {error}"),
    };

    assert!(rendered.contains(r#""observed_at_ms": 77"#));
}
