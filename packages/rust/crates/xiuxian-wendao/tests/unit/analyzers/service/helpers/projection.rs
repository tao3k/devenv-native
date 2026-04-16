use crate::analyzers::{projection_page_lookup, projection_pages_for};

use super::fixtures::analysis_fixture;

#[test]
fn projection_lookup_collects_page_ids_for_each_anchor() {
    let analysis = analysis_fixture();
    let lookup = projection_page_lookup(&analysis);

    assert!(projection_pages_for("mod-a", &lookup).is_some());
    assert!(projection_pages_for("sym-a", &lookup).is_some());
    assert!(projection_pages_for("ex-a", &lookup).is_some());
    assert!(projection_pages_for("doc-a", &lookup).is_some());
}
