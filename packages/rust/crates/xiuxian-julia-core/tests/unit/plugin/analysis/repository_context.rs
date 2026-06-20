#[test]
fn load_modelica_repository_context_prefers_source_hint_for_nested_root_package() -> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("Modelica/package.mo").as_path(),
        "within ;\npackage Modelica\nend Modelica;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Modelica/Blocks.mo").as_path(),
        "within Modelica;\nmodel Blocks\nend Blocks;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("ModelicaServices/package.mo").as_path(),
        "within ;\npackage ModelicaServices\nend ModelicaServices;\n",
    )?;

    let context = load_modelica_repository_context_for_source(
        &analysis_context("mcl", tempdir.path()).repository,
        tempdir.path(),
        "Modelica/Blocks.mo",
    )?;

    assert_eq!(context.package_root, tempdir.path().join("Modelica"));
    assert_eq!(context.root_package_name, "Modelica");
    assert_eq!(context.path_prefix.as_deref(), Some("Modelica"));
    Ok(())
}

#[test]
fn analyze_repository_preserves_import_backed_package_attributes() -> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("Modelica/package.mo").as_path(),
        "within ;\npackage Modelica\nend Modelica;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Modelica/Blocks/package.mo").as_path(),
        "within Modelica;\npackage Blocks\n  import SI = Modelica.Units.SI;\n  import Modelica.Math;\n  import Modelica.Math.*;\nend Blocks;\n",
    )?;

    let output = analyze_repository(&analysis_context("mcl", tempdir.path()), tempdir.path())?;
    let payload = output
        .imports
        .iter()
        .map(|import| {
            json!({
                "module_id": import.module_id,
                "import_name": import.import_name,
                "target_package": import.target_package,
                "source_module": import.source_module,
                "kind": format!("{:?}", import.kind),
                "line_start": import.line_start,
                "resolved_id": import.resolved_id,
                "attributes": import.attributes,
            })
        })
        .collect::<Vec<_>>();

    assert_sorted_json_snapshot(
        "analyze_repository_preserves_import_backed_package_attributes",
        payload
    );
    Ok(())
}
