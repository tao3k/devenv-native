use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_wendao::{Entity, EntityType, Relation, RelationType};

#[test]
fn test_multi_hop_search() {
    let graph = KnowledgeGraph::new();

    let entities = vec![
        ("A", EntityType::Concept),
        ("B", EntityType::Concept),
        ("C", EntityType::Concept),
        ("D", EntityType::Concept),
    ];

    for (name, etype) in &entities {
        let entity = Entity::new(
            format!("concept:{name}"),
            name.to_string(),
            etype.clone(),
            format!("Concept {name}"),
        );
        assert!(graph.add_entity(entity).is_ok());
    }

    for i in 0..entities.len() - 1 {
        let relation = Relation::new(
            entities[i].0.to_string(),
            entities[i + 1].0.to_string(),
            RelationType::RelatedTo,
            "Related".to_string(),
        );
        assert!(graph.add_relation(relation).is_ok());
    }

    let results = graph.multi_hop_search("A", 2);
    assert!(results.len() >= 2);

    let results = graph.multi_hop_search("A", 3);
    assert!(results.len() >= 3);
}

#[test]
fn test_multi_hop_search_bidirectional() {
    let graph = KnowledgeGraph::new();

    // Create: A -> B -> C, D -> B (D points TO B, not from B)
    for name in &["A", "B", "C", "D"] {
        let entity = Entity::new(
            format!("concept:{name}"),
            name.to_string(),
            EntityType::Concept,
            format!("Concept {name}"),
        );
        assert!(graph.add_entity(entity).is_ok());
    }

    // A -> B
    assert!(
        graph
            .add_relation(Relation::new(
                "A".to_string(),
                "B".to_string(),
                RelationType::RelatedTo,
                "A to B".to_string(),
            ))
            .is_ok()
    );

    // B -> C
    assert!(
        graph
            .add_relation(Relation::new(
                "B".to_string(),
                "C".to_string(),
                RelationType::RelatedTo,
                "B to C".to_string(),
            ))
            .is_ok()
    );

    // D -> B (D points to B; from B's perspective this is an incoming edge)
    assert!(
        graph
            .add_relation(Relation::new(
                "D".to_string(),
                "B".to_string(),
                RelationType::DependsOn,
                "D depends on B".to_string(),
            ))
            .is_ok()
    );

    // From B with 2 hops: should reach A (via incoming), C (via outgoing), D (via incoming)
    let results = graph.multi_hop_search("B", 2);
    let names: Vec<String> = results.iter().map(|e| e.name.clone()).collect();

    assert!(
        names.contains(&"B".to_string()),
        "Start entity should be included. Got: {names:?}",
    );
    assert!(
        names.contains(&"C".to_string()),
        "Outgoing neighbor C should be found. Got: {names:?}",
    );
    assert!(
        names.contains(&"D".to_string()),
        "Incoming neighbor D should be found via bidirectional traversal. Got: {names:?}",
    );
    assert!(
        names.contains(&"A".to_string()),
        "Incoming neighbor A should be found via bidirectional traversal. Got: {names:?}",
    );
}
