use xiuxian_qianji::contracts::{EdgeDefinition, NodeDefinition, NodeTaskType, QianjiManifest};

#[test]
fn test_qianji_manifest_schema_contract() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = QianjiManifest {
        name: "Test_Schema".to_string(),
        nodes: vec![NodeDefinition {
            id: "A".to_string(),
            task_type: NodeTaskType::new("mock"),
            weight: 1.5,
            params: serde_json::json!({"key": "value"}),
            contract: None,
            method: None,
            path: None,
            base_url: None,
            query: None,
            argv: None,
            annotation: None,
            llm: None,
            consensus: None,
        }],
        edges: vec![EdgeDefinition {
            from: "A".to_string(),
            to: "B".to_string(),
            label: Some("branch_1".to_string()),
            weight: 1.0,
        }],
    };

    let raw = serde_json::to_value(&manifest)?;

    // Validate Schema/Contract Shape
    assert_eq!(raw["name"], "Test_Schema");

    // Nodes
    let Some(nodes) = raw["nodes"].as_array() else {
        return Err(std::io::Error::other("nodes must be an array").into());
    };
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["id"], "A");
    assert_eq!(nodes[0]["task_type"], "mock");
    assert_eq!(nodes[0]["weight"], 1.5);
    assert_eq!(nodes[0]["params"]["key"], "value");

    // Edges
    let Some(edges) = raw["edges"].as_array() else {
        return Err(std::io::Error::other("edges must be an array").into());
    };
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["from"], "A");
    assert_eq!(edges[0]["to"], "B");
    assert_eq!(edges[0]["label"], "branch_1");
    assert_eq!(edges[0]["weight"], 1.0);
    Ok(())
}
