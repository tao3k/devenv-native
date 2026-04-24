use xiuxian_wendao_core::WendaoResourceUri;

#[test]
fn parse_resource_uri_normalizes_canonical_shape() {
    let Ok(parsed) = WendaoResourceUri::parse(
        "  wendao://skills/Agenda-Management/references/flow/main.md#frag  ",
    ) else {
        panic!("URI should parse");
    };

    assert_eq!(parsed.semantic_name(), "agenda-management");
    assert_eq!(parsed.entity_name(), "flow/main.md");
    assert_eq!(
        parsed.canonical_uri(),
        "wendao://skills/agenda-management/references/flow/main.md"
    );
    assert_eq!(
        parsed.entity_relative_path().to_string_lossy(),
        "flow/main.md"
    );
    assert_eq!(
        parsed.candidate_paths(),
        vec![std::path::PathBuf::from("flow/main.md")]
    );
}

#[test]
fn parse_resource_uri_rejects_missing_extension_and_traversal() {
    let missing_extension =
        match WendaoResourceUri::parse("wendao://skills/demo/references/flow/main") {
            Ok(parsed) => panic!("missing-extension URI unexpectedly parsed: {parsed:?}"),
            Err(error) => error,
        };
    assert!(
        missing_extension
            .to_string()
            .contains("must include a file extension")
    );

    let traversal = match WendaoResourceUri::parse("wendao://skills/demo/references/../secret.md") {
        Ok(parsed) => panic!("traversal URI unexpectedly parsed: {parsed:?}"),
        Err(error) => error,
    };
    assert!(traversal.to_string().contains("invalid entity path"));
}

#[test]
fn parse_resource_uri_rejects_unknown_resource_kind() {
    let error = match WendaoResourceUri::parse(
        "wendao://unknown/daemon/references/session/dispatch.toml",
    ) {
        Ok(parsed) => panic!("unknown resource kind URI unexpectedly parsed: {parsed:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("invalid wendao resource URI"));
}
