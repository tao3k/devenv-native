use std::collections::BTreeSet;

use crate::analyzers::{ModuleRecord, RelationKind, RelationRecord, SymbolRecord};

pub(crate) fn example_relation_lookup(relations: &[RelationRecord]) -> BTreeSet<(String, String)> {
    relations
        .iter()
        .filter(|relation| relation.kind == RelationKind::ExampleOf)
        .map(|relation| (relation.source_id.clone(), relation.target_id.clone()))
        .collect()
}

pub(crate) fn related_symbols_for_example(
    example_id: &str,
    relation_lookup: &BTreeSet<(String, String)>,
    symbols: &[SymbolRecord],
) -> Vec<String> {
    let symbol_ids = relation_lookup
        .iter()
        .filter(|(source_id, _)| source_id == example_id)
        .map(|(_, target_id)| target_id.as_str())
        .collect::<BTreeSet<_>>();

    symbols
        .iter()
        .filter(|symbol| symbol_ids.contains(symbol.symbol_id.as_str()))
        .flat_map(|symbol| {
            [
                symbol.name.to_ascii_lowercase(),
                symbol.qualified_name.to_ascii_lowercase(),
            ]
        })
        .collect()
}

pub(crate) fn related_modules_for_example(
    example_id: &str,
    relation_lookup: &BTreeSet<(String, String)>,
    modules: &[ModuleRecord],
) -> Vec<String> {
    let module_ids = relation_lookup
        .iter()
        .filter(|(source_id, _)| source_id == example_id)
        .map(|(_, target_id)| target_id.as_str())
        .collect::<BTreeSet<_>>();

    modules
        .iter()
        .filter(|module| module_ids.contains(module.module_id.as_str()))
        .flat_map(|module| {
            let short_name = module
                .qualified_name
                .rsplit('.')
                .next()
                .unwrap_or(module.qualified_name.as_str())
                .to_ascii_lowercase();
            [module.qualified_name.to_ascii_lowercase(), short_name]
        })
        .collect()
}
