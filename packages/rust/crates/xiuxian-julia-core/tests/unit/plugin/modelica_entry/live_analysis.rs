#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_emits_modelica_module_and_symbols() -> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "Controllers/PI.mo".to_string(),
            contents: "within Demo.Controllers;\nmodel PI\n  parameter Real k = 1;\n  parameter Real Ti = 0.1;\n  Real y;\nequation\n  y = k;\nend PI;\n".to_string(),
        },
    )?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "Controllers/PI.mo" && module.qualified_name == "PI")
    );
    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "Controllers/PI.mo"
                && symbol.qualified_name == "PI"
                && symbol.name == "PI"
                && symbol.module_id.as_deref() == Some("repo:demo:module:PI")
        }),
        "symbols: {:?}",
        output.symbols
    );
    let model = output
        .symbols
        .iter()
        .find(|symbol| symbol.name == "PI")
        .unwrap_or_else(|| panic!("symbols: {:?}", output.symbols));
    assert!(
        model
            .attributes
            .get("class_name")
            .is_some_and(|value| value == "PI"),
        "model attrs: {:?}",
        model.attributes
    );
    assert!(
        model
            .attributes
            .get("restriction")
            .is_some_and(|value| value == "model"),
        "model attrs: {:?}",
        model.attributes
    );
    assert!(
        model
            .attributes
            .get("top_level")
            .is_some_and(|value| value == "true"),
        "model attrs: {:?}",
        model.attributes
    );
    Ok(())
}

#[test]
fn analyze_file_supports_modelica_standard_library_package_via_process_managed_parser_summary()
-> TestResult {
    if std::env::var_os("RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST").is_none() {
        eprintln!("skipping process-managed Modelica analyze_file proof");
        return Ok(());
    }

    let source_path = repo_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary/Modelica/Blocks/package.mo",
    );
    if !source_path.is_file() {
        eprintln!(
            "skipping process-managed Modelica analyze_file proof; missing {}",
            source_path.display()
        );
        return Ok(());
    }

    let tempdir = TempDir::new()?;
    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("mcl-live", tempdir.path()),
        &RepoSourceFile {
            path: "Modelica/Blocks/package.mo".to_string(),
            contents: fs::read_to_string(&source_path)?,
        },
    )?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "Modelica/Blocks/package.mo"
                && module.qualified_name == "Blocks"),
        "modules: {:?}",
        output.modules
    );
    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Modelica/Blocks/package.mo" && symbol.name == "Init"),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_supports_modelica_standard_library_leaf_via_nested_root_context() -> TestResult {
    if std::env::var_os("RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST").is_none() {
        eprintln!("skipping process-managed Modelica leaf analyze_file proof");
        return Ok(());
    }

    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let repository_root = repo_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary",
    );
    let source_path = repository_root.join("Modelica/Clocked/Types/SolverMethod.mo");
    if !source_path.is_file() {
        eprintln!(
            "skipping process-managed Modelica leaf analyze_file proof; missing {}",
            source_path.display()
        );
        return Ok(());
    }

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("mcl", repository_root.as_path()),
        &RepoSourceFile {
            path: "Modelica/Clocked/Types/SolverMethod.mo".to_string(),
            contents: fs::read_to_string(&source_path)?,
        },
    )?;

    assert!(output.modules.is_empty(), "modules: {:?}", output.modules);
    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "Modelica/Clocked/Types/SolverMethod.mo"
                && symbol.name == "SolverMethod"
                && symbol.qualified_name == "Modelica.Clocked.Types.SolverMethod"
                && symbol.module_id.as_deref() == Some("repo:mcl:module:Modelica.Clocked.Types")
        }),
        "symbols: {:?}",
        output.symbols
    );
    assert!(output.imports.is_empty(), "imports: {:?}", output.imports);
    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_uses_repository_module_context_for_safe_leaf_files() -> TestResult {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
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
            contents: "within DemoLib;\nmodel PI\nend PI;\n".to_string(),
        },
    )?;

    assert!(output.modules.is_empty(), "modules: {:?}", output.modules);
    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "PI.mo"
                && symbol.qualified_name == "DemoLib.PI"
                && symbol.module_id.as_deref() == Some("repo:demo:module:DemoLib")
        }),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}
