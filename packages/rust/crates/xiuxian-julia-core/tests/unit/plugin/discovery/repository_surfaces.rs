#[test]
fn repository_snapshot_preloads_modelica_entries_and_package_orders() {
    let tempdir = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )
    .unwrap_or_else(|error| panic!("write root package: {error}"));
    fs::create_dir_all(tempdir.path().join("Blocks"))
        .unwrap_or_else(|error| panic!("create Blocks dir: {error}"));
    fs::write(
        tempdir.path().join("Blocks/package.mo"),
        "within DemoLib;\npackage Blocks\nend Blocks;\n",
    )
    .unwrap_or_else(|error| panic!("write nested package: {error}"));
    fs::write(
        tempdir.path().join("Blocks/package.order"),
        "Interfaces\nUtilities\n",
    )
    .unwrap_or_else(|error| panic!("write package.order: {error}"));
    fs::write(tempdir.path().join("README.md"), "# Demo\n")
        .unwrap_or_else(|error| panic!("write readme: {error}"));

    let snapshot = RepositorySnapshot::load(tempdir.path())
        .unwrap_or_else(|error| panic!("load snapshot: {error}"));
    let payload = json!({
        "entries": snapshot
            .entries()
            .iter()
            .map(|entry| json!({
                "relative_path": entry.relative_path,
                "surface": surface_name(entry.surface),
                "has_modelica_contents": entry.modelica_contents.is_some(),
            }))
            .collect::<Vec<_>>(),
        "package_orders": snapshot.package_orders(),
        "package_files": snapshot
            .package_files()
            .unwrap_or_else(|error| panic!("package files: {error}"))
            .into_iter()
            .map(|entry| entry.relative_path.clone())
            .collect::<Vec<_>>(),
    });

    assert_sorted_json_snapshot(
        "repository_snapshot_preloads_modelica_entries_and_package_orders",
        payload
    );
}

#[test]
fn module_sort_key_uses_package_order_hierarchy() {
    let orders = BTreeMap::from([
        (
            String::new(),
            vec!["UsersGuide".to_string(), "Controllers".to_string()],
        ),
        (
            "Controllers".to_string(),
            vec!["Examples".to_string(), "PI".to_string()],
        ),
    ]);
    let payload = json!([
        {
            "path": "package.mo",
            "key": module_sort_key("package.mo", &orders),
        },
        {
            "path": "UsersGuide/package.mo",
            "key": module_sort_key("UsersGuide/package.mo", &orders),
        },
        {
            "path": "Controllers/package.mo",
            "key": module_sort_key("Controllers/package.mo", &orders),
        },
        {
            "path": "Controllers/Examples/package.mo",
            "key": module_sort_key("Controllers/Examples/package.mo", &orders),
        },
    ]);

    assert_sorted_json_snapshot("module_sort_key_uses_package_order_hierarchy", payload);
}

#[test]
fn example_sort_key_uses_package_order_leaf_entries() {
    let orders = BTreeMap::from([
        (String::new(), vec!["Controllers".to_string()]),
        ("Controllers".to_string(), vec!["Examples".to_string()]),
        (
            "Controllers/Examples".to_string(),
            vec!["Step".to_string(), "Alpha".to_string()],
        ),
    ]);
    let payload = json!([
        {
            "path": "Controllers/Examples/Step.mo",
            "key": example_sort_key("Controllers/Examples/Step.mo", &orders),
        },
        {
            "path": "Controllers/Examples/Alpha.mo",
            "key": example_sort_key("Controllers/Examples/Alpha.mo", &orders),
        },
    ]);

    assert_sorted_json_snapshot("example_sort_key_uses_package_order_leaf_entries", payload);
}

#[test]
fn detects_repository_surfaces() {
    let payload = json!([
        {
            "path": "Controllers/Examples/Step.mo",
            "surface": surface_name(repository_surface("Controllers/Examples/Step.mo")),
        },
        {
            "path": "Controllers/Examples/ExampleUtilities/Helper.mo",
            "surface": surface_name(repository_surface(
                "Controllers/Examples/ExampleUtilities/Helper.mo",
            )),
        },
        {
            "path": "Controllers/Examples/Utilities/Helper.mo",
            "surface": surface_name(repository_surface("Controllers/Examples/Utilities/Helper.mo")),
        },
        {
            "path": "Controllers/Internal/Helper.mo",
            "surface": surface_name(repository_surface("Controllers/Internal/Helper.mo")),
        },
        {
            "path": "Controllers/PI.mo",
            "surface": surface_name(repository_surface("Controllers/PI.mo")),
        },
        {
            "path": "UsersGuide/Overview.mo",
            "surface": surface_name(repository_surface("UsersGuide/Overview.mo")),
        },
    ]);

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_json_snapshot!("detects_repository_surfaces", payload);
    });
}
