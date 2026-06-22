const USERS_GUIDE_DOC_PATHS: &[&str] = &[
    "Controllers/UsersGuide/package.mo",
    "Controllers/UsersGuide/Conventions.mo",
    "Controllers/UsersGuide/Connectors.mo",
    "Controllers/UsersGuide/Implementation.mo",
    "Controllers/UsersGuide/RevisionHistory.mo",
    "Controllers/UsersGuide/VersionManagement.mo",
    "Controllers/UsersGuide/Tutorial/package.mo",
    "Controllers/UsersGuide/Tutorial/FirstSteps.mo",
    "Controllers/UsersGuide/ReleaseNotes.mo",
    "Controllers/UsersGuide/References.mo",
    "Controllers/UsersGuide/Contact.mo",
    "Controllers/UsersGuide/Concept.mo",
    "Controllers/UsersGuide/Parameters.mo",
    "UsersGuide/Overview.mo",
    "UsersGuide/Conventions.mo",
    "UsersGuide/Connectors.mo",
    "UsersGuide/Implementation.mo",
    "UsersGuide/RevisionHistory.mo",
    "UsersGuide/VersionManagement.mo",
    "UsersGuide/Literature.mo",
    "UsersGuide/Glossar.mo",
    "UsersGuide/Parameterization.mo",
];

fn users_guide_doc_formats_payload() -> Vec<serde_json::Value> {
    USERS_GUIDE_DOC_PATHS
        .iter()
        .map(|path| {
            json!({
                "path": path,
                "file_format": doc_format_hint(path, false),
                "annotation_format": doc_format_hint(path, true),
            })
        })
        .collect()
}

#[test]
fn infers_users_guide_doc_formats() {
    let payload = json!(users_guide_doc_formats_payload());

    assert_sorted_json_snapshot("infers_users_guide_doc_formats", payload);
}

#[test]
fn detects_nested_users_guide_topics_from_conventions_files() {
    let payload = json!({
        "conventions": documented_nested_users_guide_topics(
            "package Conventions\n  package Documentation\n    annotation (Documentation(info=\"<html>Doc.</html>\"));\n  end Documentation;\n  package ModelicaCode\n    annotation (Documentation(info=\"<html>Code.</html>\"));\n  end ModelicaCode;\n  class Icons\n    annotation (Documentation(info=\"<html>Icons.</html>\"));\n  end Icons;\nend Conventions;\n"
        )
        .into_iter()
        .map(|topic| json!({
            "title": topic.title,
            "format": topic.format,
        }))
        .collect::<Vec<_>>(),
        "non_conventions": documented_nested_users_guide_topics(
            "model Overview\n  annotation (Documentation(info=\"<html>Overview.</html>\"));\nend Overview;\n"
        )
        .into_iter()
        .map(|topic| json!({
            "title": topic.title,
            "format": topic.format,
        }))
        .collect::<Vec<_>>(),
    });

    assert_sorted_json_snapshot(
        "detects_nested_users_guide_topics_from_conventions_files",
        payload
    );
}

#[test]
fn detects_release_notes_topics_from_nested_release_notes_files() {
    let payload = json!({
        "release_notes": documented_release_notes_topics(
            "package ReleaseNotes\n  class VersionManagement\n    annotation (Documentation(info=\"<html>Version workflow.</html>\"));\n  end VersionManagement;\n  class Version_4_1_0\n    annotation (Documentation(info=\"<html>Release 4.1.0.</html>\"));\n  end Version_4_1_0;\n  class Version_4_0_0\n    annotation (Documentation(info=\"<html>Release 4.0.0.</html>\"));\n  end Version_4_0_0;\nend ReleaseNotes;\n"
        )
        .into_iter()
        .map(|topic| json!({
            "title": topic.title,
            "format": topic.format,
        }))
        .collect::<Vec<_>>(),
        "generic_page": documented_release_notes_topics(
            "model Overview\n  annotation (Documentation(info=\"<html>Overview.</html>\"));\nend Overview;\n"
        )
        .into_iter()
        .map(|topic| json!({
            "title": topic.title,
            "format": topic.format,
        }))
        .collect::<Vec<_>>(),
    });

    assert_sorted_json_snapshot(
        "detects_release_notes_topics_from_nested_release_notes_files",
        payload
    );
}
