#[test]
fn infers_users_guide_doc_formats() {
    let payload = json!([
        {
            "path": "Controllers/UsersGuide/package.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/package.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/package.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Conventions.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Conventions.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Conventions.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Connectors.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Connectors.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Connectors.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Implementation.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Implementation.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Implementation.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/RevisionHistory.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/RevisionHistory.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/RevisionHistory.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/VersionManagement.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/VersionManagement.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/VersionManagement.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Tutorial/package.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Tutorial/package.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Tutorial/package.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Tutorial/FirstSteps.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Tutorial/FirstSteps.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Tutorial/FirstSteps.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/ReleaseNotes.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/ReleaseNotes.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/ReleaseNotes.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/References.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/References.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/References.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Contact.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Contact.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Contact.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Concept.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Concept.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Concept.mo", true),
        },
        {
            "path": "Controllers/UsersGuide/Parameters.mo",
            "file_format": doc_format_hint("Controllers/UsersGuide/Parameters.mo", false),
            "annotation_format": doc_format_hint("Controllers/UsersGuide/Parameters.mo", true),
        },
        {
            "path": "UsersGuide/Overview.mo",
            "file_format": doc_format_hint("UsersGuide/Overview.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/Overview.mo", true),
        },
        {
            "path": "UsersGuide/Conventions.mo",
            "file_format": doc_format_hint("UsersGuide/Conventions.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/Conventions.mo", true),
        },
        {
            "path": "UsersGuide/Connectors.mo",
            "file_format": doc_format_hint("UsersGuide/Connectors.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/Connectors.mo", true),
        },
        {
            "path": "UsersGuide/Implementation.mo",
            "file_format": doc_format_hint("UsersGuide/Implementation.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/Implementation.mo", true),
        },
        {
            "path": "UsersGuide/RevisionHistory.mo",
            "file_format": doc_format_hint("UsersGuide/RevisionHistory.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/RevisionHistory.mo", true),
        },
        {
            "path": "UsersGuide/VersionManagement.mo",
            "file_format": doc_format_hint("UsersGuide/VersionManagement.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/VersionManagement.mo", true),
        },
        {
            "path": "UsersGuide/Literature.mo",
            "file_format": doc_format_hint("UsersGuide/Literature.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/Literature.mo", true),
        },
        {
            "path": "UsersGuide/Glossar.mo",
            "file_format": doc_format_hint("UsersGuide/Glossar.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/Glossar.mo", true),
        },
        {
            "path": "UsersGuide/Parameterization.mo",
            "file_format": doc_format_hint("UsersGuide/Parameterization.mo", false),
            "annotation_format": doc_format_hint("UsersGuide/Parameterization.mo", true),
        },
    ]);

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
