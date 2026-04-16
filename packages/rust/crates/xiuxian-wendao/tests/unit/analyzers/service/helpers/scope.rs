use std::collections::BTreeSet;

use crate::analyzers::service::{
    docs_in_scope, documented_symbol_ids, resolve_module_scope, symbols_in_scope,
};

use super::fixtures::{analysis_fixture, some_or_panic};

#[test]
fn scope_helpers_filter_docs_symbols_and_modules() {
    let analysis = analysis_fixture();
    let scoped_module = some_or_panic(
        resolve_module_scope(Some("alpha.beta"), &analysis.modules),
        "module scope should resolve by qualified name",
    );
    assert_eq!(scoped_module.module_id, "mod-a");
    assert_eq!(
        some_or_panic(
            resolve_module_scope(Some("src/alpha/beta.rs"), &analysis.modules),
            "module scope should resolve by path",
        )
        .module_id,
        "mod-a"
    );

    let scoped_symbols = symbols_in_scope(Some(scoped_module), &analysis.symbols);
    assert_eq!(
        scoped_symbols
            .iter()
            .map(|symbol| symbol.symbol_id.as_str())
            .collect::<Vec<_>>(),
        vec!["sym-a"]
    );

    let scoped_docs = docs_in_scope(Some(scoped_module), &analysis);
    assert_eq!(
        scoped_docs
            .iter()
            .map(|doc| doc.doc_id.as_str())
            .collect::<Vec<_>>(),
        vec!["doc-a", "doc-b"]
    );

    let documented =
        documented_symbol_ids(Some(scoped_module), &analysis.symbols, &analysis.relations);
    assert_eq!(documented, BTreeSet::from([String::from("sym-a")]));
}
