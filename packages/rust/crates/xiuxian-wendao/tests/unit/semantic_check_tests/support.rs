use super::*;

pub(super) fn create_node_with_observations(
    node_id: &str,
    observations: Vec<CodeObservation>,
) -> PageIndexNode {
    PageIndexNode {
        node_id: node_id.to_string(),
        parent_id: None,
        title: "Test Node".to_string(),
        level: 1,
        text: Arc::from(""),
        summary: None,
        children: Vec::new(),
        blocks: Vec::new(),
        metadata: PageIndexMeta {
            line_range: (1, 10),
            byte_range: Some((0, 100)),
            structural_path: vec!["Test".to_string()],
            content_hash: Some("abc123".to_string()),
            attributes: std::collections::HashMap::new(),
            token_count: 10,
            is_thinned: false,
            logbook: Vec::new(),
            observations,
        },
    }
}

pub(super) fn parse_observation(raw: &str) -> CodeObservation {
    let Some(observation) = CodeObservation::parse(raw) else {
        panic!("expected test observation to parse: {raw}");
    };
    observation
}
