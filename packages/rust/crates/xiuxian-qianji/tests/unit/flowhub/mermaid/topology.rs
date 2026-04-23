use crate::contracts::FlowhubGraphTopology;
use crate::flowhub::{analyze_mermaid_flowchart_topology, parse_mermaid_flowchart};

#[test]
fn classifies_acyclic_flowchart_as_dag() {
    let parsed = parse_mermaid_flowchart(
        "flowchart LR\n  A[\"wendao\"] --> B[\"page lookup\"]\n  B --> C[\"done gate\"]\n",
        "wendao-page",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("dag flowchart should parse: {error}"));

    let analysis = analyze_mermaid_flowchart_topology(&parsed);
    assert_eq!(analysis.topology, FlowhubGraphTopology::Dag);
    assert!(analysis.cyclic_components.is_empty());
}

#[test]
fn classifies_cycle_with_exit_as_bounded_loop() {
    let parsed = parse_mermaid_flowchart(
        "flowchart LR\n  A[\"wendao\"] --> B[\"search step\"]\n  B --> C[\"page lookup\"]\n  C --> D[\"done gate\"]\n  B --> E[\"diagnostics\"]\n  E --> B\n",
        "wendao-search",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("bounded-loop flowchart should parse: {error}"));

    let analysis = analyze_mermaid_flowchart_topology(&parsed);
    assert_eq!(analysis.topology, FlowhubGraphTopology::BoundedLoop);
    assert_eq!(
        analysis.cyclic_components,
        vec![vec!["diagnostics".to_string(), "search step".to_string()]]
    );
}

#[test]
fn classifies_cycle_without_exit_as_open_loop() {
    let parsed = parse_mermaid_flowchart(
        "flowchart LR\n  A[\"wendao\"] --> B[\"search step\"]\n  B --> C[\"diagnostics\"]\n  C --> B\n",
        "wendao-search",
        &["wendao".to_string()],
    )
    .unwrap_or_else(|error| panic!("open-loop flowchart should parse: {error}"));

    let analysis = analyze_mermaid_flowchart_topology(&parsed);
    assert_eq!(analysis.topology, FlowhubGraphTopology::OpenLoop);
    assert_eq!(
        analysis.cyclic_components,
        vec![vec!["diagnostics".to_string(), "search step".to_string()]]
    );
}
