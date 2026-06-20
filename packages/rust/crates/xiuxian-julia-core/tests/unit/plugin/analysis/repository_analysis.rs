#[test]
fn analyze_repository_keeps_top_level_package_paths() -> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Example.mo").as_path(),
        "within DemoLib;\nmodel Example\nend Example;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "package.mo" && module.qualified_name == "DemoLib")
    );
    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Example.mo" && symbol.qualified_name == "DemoLib.Example")
    );
    assert_eq!(output.diagnostics[0].path, "package.mo");
    Ok(())
}

#[test]
fn analyze_repository_lexically_collects_safe_root_package_imports_and_docs() -> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\n  import SI = Modelica.Units.SI;\n  import Modelica.Math;\n  import Modelica.Math.*;\n  annotation(Documentation(info = \"doc\"));\nend DemoLib;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "package.mo" && module.qualified_name == "DemoLib"),
        "modules: {:?}",
        output.modules
    );
    assert!(output.symbols.is_empty(), "symbols: {:?}", output.symbols);
    assert!(
        output.imports.iter().any(|import| {
            import.path == "package.mo"
                && import.module_id == "repo:demo:module:DemoLib"
                && import.import_name == "SI"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Units.SI"
        }),
        "imports: {:?}",
        output.imports
    );
    assert!(
        output.imports.iter().any(|import| {
            import.path == "package.mo"
                && import.module_id == "repo:demo:module:DemoLib"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        output.imports
    );
    assert!(
        output
            .docs
            .iter()
            .any(|doc| doc.path == "package.mo#annotation.documentation"),
        "docs: {:?}",
        output.docs
    );
    assert!(
        output.relations.iter().any(|relation| {
            relation.source_id == "repo:demo:doc:package.mo#annotation.documentation"
                && relation.target_id == "repo:demo:module:DemoLib"
        }),
        "relations: {:?}",
        output.relations
    );
    Ok(())
}

#[test]
fn analyze_repository_preserves_root_package_nested_declarations_via_parser_summary_fallback()
-> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\n  model Controller\n  end Controller;\nend DemoLib;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "package.mo" && symbol.qualified_name == "DemoLib.Controller"
        }),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}

#[test]
fn analyze_repository_lexically_collects_safe_nested_package_imports_and_docs() -> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Blocks/package.mo").as_path(),
        "within DemoLib;\npackage Blocks\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend Blocks;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output.modules.iter().any(|module| {
            module.path == "Blocks/package.mo"
                && module.qualified_name == "DemoLib.Blocks"
                && module.module_id == "repo:demo:module:DemoLib.Blocks"
        }),
        "modules: {:?}",
        output.modules
    );
    assert!(
        !output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Blocks/package.mo"),
        "symbols: {:?}",
        output.symbols
    );
    assert!(
        output.imports.iter().any(|import| {
            import.path == "Blocks/package.mo"
                && import.module_id == "repo:demo:module:DemoLib.Blocks"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        output.imports
    );
    assert!(
        output
            .docs
            .iter()
            .any(|doc| doc.path == "Blocks/package.mo#annotation.documentation"),
        "docs: {:?}",
        output.docs
    );
    assert!(
        output.relations.iter().any(|relation| {
            relation.source_id == "repo:demo:doc:Blocks/package.mo#annotation.documentation"
                && relation.target_id == "repo:demo:module:DemoLib.Blocks"
        }),
        "relations: {:?}",
        output.relations
    );
    Ok(())
}

#[test]
fn analyze_repository_preserves_nested_package_declarations_via_parser_summary_fallback()
-> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Blocks/package.mo").as_path(),
        "within DemoLib;\npackage Blocks\n  model Controller\n  end Controller;\nend Blocks;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "Blocks/package.mo"
                && symbol.qualified_name == "DemoLib.Blocks.Controller"
        }),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}

#[test]
fn analyze_repository_supports_dominant_nested_root_package() -> TestResult {
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

    preflight_repository(&analysis_context("mcl", tempdir.path()), tempdir.path())?;
    let output = analyze_repository(&analysis_context("mcl", tempdir.path()), tempdir.path())?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "Modelica/package.mo"
                && module.qualified_name == "Modelica")
    );
    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Modelica/Blocks.mo"
                && symbol.qualified_name == "Modelica.Blocks")
    );
    assert_eq!(output.diagnostics[0].path, "Modelica/package.mo");
    Ok(())
}
