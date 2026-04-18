#[test]
fn normalizes_synthetic_section_titles() {
    let payload = json!([
        {
            "raw": "Documentation",
            "title": synthetic_section_title("Documentation"),
        },
        {
            "raw": "ModelicaCode",
            "title": synthetic_section_title("ModelicaCode"),
        },
        {
            "raw": "VersionManagement",
            "title": synthetic_section_title("VersionManagement"),
        },
        {
            "raw": "Version_4_1_0",
            "title": synthetic_section_title("Version_4_1_0"),
        },
    ]);

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_json_snapshot!("normalizes_synthetic_section_titles", payload);
    });
}

#[test]
fn doc_sort_key_uses_package_order_and_annotation_position() {
    let orders = BTreeMap::from([
        (String::new(), vec!["Controllers".to_string()]),
        ("Controllers".to_string(), vec!["UsersGuide".to_string()]),
        (
            "Controllers/UsersGuide".to_string(),
            vec![
                "Tutorial".to_string(),
                "References".to_string(),
                "ReleaseNotes".to_string(),
                "Tuning".to_string(),
            ],
        ),
        (
            "Controllers/UsersGuide/Tutorial".to_string(),
            vec!["FirstSteps".to_string()],
        ),
    ]);
    let payload = json!([
        {
            "path": "Controllers/UsersGuide/package.mo",
            "key": doc_sort_key("Controllers/UsersGuide/package.mo", &orders),
        },
        {
            "path": "Controllers/UsersGuide/Tutorial/package.mo",
            "key": doc_sort_key("Controllers/UsersGuide/Tutorial/package.mo", &orders),
        },
        {
            "path": "Controllers/UsersGuide/Tutorial/FirstSteps.mo",
            "key": doc_sort_key("Controllers/UsersGuide/Tutorial/FirstSteps.mo", &orders),
        },
        {
            "path": "Controllers/UsersGuide/Tutorial/FirstSteps.mo#annotation.documentation",
            "key": doc_sort_key(
                "Controllers/UsersGuide/Tutorial/FirstSteps.mo#annotation.documentation",
                &orders,
            ),
        },
        {
            "path": "Controllers/UsersGuide/Conventions.mo#section.Documentation",
            "key": doc_sort_key(
                "Controllers/UsersGuide/Conventions.mo#section.Documentation",
                &orders,
            ),
        },
        {
            "path": "Controllers/UsersGuide/References.mo",
            "key": doc_sort_key("Controllers/UsersGuide/References.mo", &orders),
        },
        {
            "path": "Controllers/UsersGuide/ReleaseNotes.mo#section.VersionManagement",
            "key": doc_sort_key(
                "Controllers/UsersGuide/ReleaseNotes.mo#section.VersionManagement",
                &orders,
            ),
        },
        {
            "path": "Controllers/UsersGuide/ReleaseNotes.mo",
            "key": doc_sort_key("Controllers/UsersGuide/ReleaseNotes.mo", &orders),
        },
        {
            "path": "Controllers/UsersGuide/Tuning.mo",
            "key": doc_sort_key("Controllers/UsersGuide/Tuning.mo", &orders),
        },
    ]);

    assert_sorted_json_snapshot(
        "doc_sort_key_uses_package_order_and_annotation_position",
        payload
    );
}

#[test]
fn filters_supported_users_guide_doc_assets() {
    let payload = json!([
        {
            "path": "UsersGuide/package.mo",
            "supported": is_supported_users_guide_doc_path(Path::new("UsersGuide/package.mo")),
        },
        {
            "path": "UsersGuide/Overview.mo",
            "supported": is_supported_users_guide_doc_path(Path::new("UsersGuide/Overview.mo")),
        },
        {
            "path": "UsersGuide/Guide.md",
            "supported": is_supported_users_guide_doc_path(Path::new("UsersGuide/Guide.md")),
        },
        {
            "path": "UsersGuide/package.order",
            "supported": is_supported_users_guide_doc_path(Path::new("UsersGuide/package.order")),
        },
    ]);

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_json_snapshot!("filters_supported_users_guide_doc_assets", payload);
    });
}

#[test]
fn normalizes_doc_titles_from_paths() {
    let payload = json!([
        {
            "path": "README.md",
            "title": doc_title(Path::new("README.md")),
        },
        {
            "path": "UsersGuide/package.mo",
            "title": doc_title(Path::new("UsersGuide/package.mo")),
        },
        {
            "path": "UsersGuide/Overview.mo",
            "title": doc_title(Path::new("UsersGuide/Overview.mo")),
        },
        {
            "path": "UsersGuide/Guide.md",
            "title": doc_title(Path::new("UsersGuide/Guide.md")),
        },
    ]);

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_json_snapshot!("normalizes_doc_titles_from_paths", payload);
    });
}
