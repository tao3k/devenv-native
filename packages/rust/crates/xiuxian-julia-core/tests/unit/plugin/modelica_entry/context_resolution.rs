#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_uses_repository_import_context_for_safe_leaf_files() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "PI.mo".to_string(),
            contents: "within DemoLib;\nmodel PI\n  import Modelica.Math;\nend PI;\n".to_string(),
        },
    )?;

    assert!(
        output.imports.iter().any(|import| {
            import.path == "PI.mo"
                && import.module_id == "repo:demo:module:DemoLib"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        output.imports
    );
    Ok(())
}

#[test]
fn analyze_file_uses_repository_root_package_context_for_package_files() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    let contents = "within ;\npackage DemoLib\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend DemoLib;\n";
    fs::write(tempdir.path().join("package.mo"), contents)?;

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "package.mo".to_string(),
            contents: contents.to_string(),
        },
    )?;

    assert!(
        output.modules.iter().any(|module| {
            module.path == "package.mo"
                && module.qualified_name == "DemoLib"
                && module.module_id == "repo:demo:module:DemoLib"
        }),
        "modules: {:?}",
        output.modules
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
        output
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.path == "package.mo" && diagnostic.severity == "info" }),
        "diagnostics: {:?}",
        output.diagnostics
    );
    Ok(())
}

#[test]
fn analyze_file_uses_repository_context_for_safe_nested_package_files() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    fs::create_dir_all(tempdir.path().join("Blocks"))?;
    let contents = "within DemoLib;\npackage Blocks\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend Blocks;\n";

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "Blocks/package.mo".to_string(),
            contents: contents.to_string(),
        },
    )?;

    assert!(
        output.modules.iter().any(|module| {
            module.path == "Blocks/package.mo"
                && module.qualified_name == "DemoLib.Blocks"
                && module.module_id == "repo:demo:module:DemoLib.Blocks"
        }),
        "modules: {:?}",
        output.modules
    );
    assert!(output.symbols.is_empty(), "symbols: {:?}", output.symbols);
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
    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_preserves_nested_package_declarations_via_parser_summary_fallback() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "Blocks/package.mo".to_string(),
            contents: "within DemoLib;\npackage Blocks\n  model Init\n  end Init;\nend Blocks;\n"
                .to_string(),
        },
    )?;

    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Blocks/package.mo" && symbol.name == "Init"),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}
