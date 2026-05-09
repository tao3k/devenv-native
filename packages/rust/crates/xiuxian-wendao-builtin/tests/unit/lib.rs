use crate::bootstrap_builtin_registry;

#[test]
fn bootstrap_builtin_registry_registers_julia_line_plugins_by_default() {
    let registry = bootstrap_builtin_registry()
        .unwrap_or_else(|error| panic!("builtin registry bootstrap should succeed: {error}"));

    assert!(
        registry.get("julia-code-parser").is_some(),
        "default builtin registry should include the external Julia plugin"
    );
    assert!(
        registry.get("modelica").is_some(),
        "builtin Julia line should also include the Modelica plugin"
    );
}
