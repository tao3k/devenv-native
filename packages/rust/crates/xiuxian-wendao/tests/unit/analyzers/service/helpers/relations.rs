use crate::analyzers::RepoBacklinkItem;
use crate::analyzers::{
    backlinks_for, documents_backlink_lookup, example_relation_lookup, related_modules_for_example,
    related_symbols_for_example,
};

use super::fixtures::analysis_fixture;

#[test]
fn backlinks_and_example_relations_are_deduplicated_and_trimmed() {
    let analysis = analysis_fixture();
    let backlink_lookup = documents_backlink_lookup(&analysis.relations, &analysis.docs);
    let (backlink_ids, backlink_items) = backlinks_for("mod-a", &backlink_lookup);
    assert_eq!(backlink_ids, Some(vec!["doc-a".to_string()]));
    assert_eq!(
        backlink_items,
        Some(vec![RepoBacklinkItem {
            id: "doc-a".to_string(),
            title: Some("Alpha Guide".to_string()),
            path: Some("docs/alpha.md".to_string()),
            kind: Some("documents".to_string()),
        }])
    );

    let relation_lookup = example_relation_lookup(&analysis.relations);
    let related_symbols = related_symbols_for_example("ex-a", &relation_lookup, &analysis.symbols);
    let related_modules = related_modules_for_example("ex-a", &relation_lookup, &analysis.modules);
    assert_eq!(
        related_symbols,
        vec!["solve".to_string(), "alpha.beta::solve".to_string()]
    );
    assert_eq!(
        related_modules,
        vec!["alpha.beta".to_string(), "beta".to_string()]
    );
}
