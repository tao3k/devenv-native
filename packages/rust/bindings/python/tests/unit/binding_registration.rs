#[path = "../../src/module.rs"]
mod module;

use pyo3::PyResult;
use xiuxian_wendao::pybindings::{
    create_knowledge_entry, dep_indexer_py::register_dependency_indexer_module,
    unified_symbol_py::register_unified_symbol_module,
};

#[test]
fn cargo_manifest_keeps_only_wendao_dependency() {
    let manifest = include_str!("../../Cargo.toml");
    assert!(manifest.contains("xiuxian-wendao"));

    for removed_dep in [
        "xiuxian-event",
        "xiuxian-types",
        "xiuxian-io",
        "xiuxian-tokenizer",
        "xiuxian-ast",
        "xiuxian-security",
        "xiuxian-code-intelligence",
        "xiuxian-edit",
        "xiuxian-vector",
        "xiuxian-tui",
        "xiuxian-memory-engine",
        "xiuxian-window",
    ] {
        assert!(
            !manifest.contains(removed_dep),
            "{removed_dep} should not remain in the binding manifest"
        );
    }
}

#[test]
fn compiles_wendao_binding_registration_paths() {
    let _schema_fn: fn(&str) -> PyResult<String> = module::py_get_schema;
    let _register_fn = module::register_module;
    let _module_fn = module::xiuxian_core_rs;
    let _create_fn = create_knowledge_entry;
    let _hash_fn: fn(&str) -> String = xiuxian_wendao::pybindings::compute_hash;
    let _dependency_indexer_fn = register_dependency_indexer_module;
    let _unified_symbol_fn = register_unified_symbol_module;
}
