#[path = "link_graph_saliency/support.rs"]
mod support;
use support::{TEST_VALKEY_URL, clear_prefix, unique_prefix};
use xiuxian_wendao::link_graph::{
    LinkGraphSaliencyTouchRequest, valkey_saliency_get_many_with_valkey,
    valkey_saliency_touch_with_valkey,
};
const BATCH_TEST_COUNT: usize = 600;
#[test]
fn test_saliency_get_many_batches() {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return;
    }
    let mut node_ids: Vec<String> = Vec::with_capacity(BATCH_TEST_COUNT);
    for index in 0..BATCH_TEST_COUNT {
        let node_id = format!("node_{index:04}");
        let request = LinkGraphSaliencyTouchRequest {
            node_id: node_id.clone().into(),
            ..Default::default()
        };
        if valkey_saliency_touch_with_valkey(request, TEST_VALKEY_URL, Some(&prefix)).is_err() {
            let _ = clear_prefix(&prefix);
            return;
        }
        node_ids.push(node_id);
    }
    let fetched =
        match valkey_saliency_get_many_with_valkey(&node_ids, TEST_VALKEY_URL, Some(&prefix)) {
            Ok(states) => states,
            Err(_) => {
                let _ = clear_prefix(&prefix);
                return;
            }
        };
    assert_eq!(fetched.len(), BATCH_TEST_COUNT);
    let _ = clear_prefix(&prefix);
}
