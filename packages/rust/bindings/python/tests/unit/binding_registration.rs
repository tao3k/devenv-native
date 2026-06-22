#[path = "../../src/module.rs"]
mod module;

use pyo3::PyResult;
use xiuxian_wendao::pybindings::{
    create_knowledge_entry, dep_indexer_py::register_dependency_indexer_module,
    unified_symbol_py::register_unified_symbol_module,
};

fn assert_registration_path_compiles<T>(_value: T) {}

#[test]
fn cargo_manifest_keeps_only_wendao_dependency() {
    let manifest = include_str!("../../Cargo.toml");
    assert!(manifest.contains("xiuxian-wendao"));

    for removed_dep in [
        "xiuxian-types",
        "xiuxian-tokenizer",
        "xiuxian-ast",
        "xiuxian-security",
        "xiuxian-code-intelligence",
        "xiuxian-edit",
        "xiuxian-vector",
        "xiuxian-tui",
    ] {
        assert!(
            !manifest.contains(removed_dep),
            "{removed_dep} should not remain in the binding manifest"
        );
    }
}

#[test]
fn compiles_wendao_binding_registration_paths() {
    assert_registration_path_compiles(module::py_get_schema as fn(&str) -> PyResult<String>);
    assert_registration_path_compiles(module::register_module);
    assert_registration_path_compiles(module::xiuxian_core_rs);
    assert_registration_path_compiles(create_knowledge_entry);
    assert_registration_path_compiles(
        xiuxian_wendao::pybindings::compute_hash as fn(&str) -> String,
    );
    assert_registration_path_compiles(register_dependency_indexer_module);
    assert_registration_path_compiles(register_unified_symbol_module);
}
