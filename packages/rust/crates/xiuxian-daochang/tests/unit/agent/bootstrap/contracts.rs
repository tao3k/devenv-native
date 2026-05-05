use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use xiuxian_daochang::XiuxianConfig;
use xiuxian_daochang::test_support::{
    init_persona_registries_internal_len, load_skill_templates_from_embedded_registry,
    resolve_notebook_root, resolve_prj_data_home, resolve_project_root, resolve_template_globs,
};
use xiuxian_qianhuan::{ManifestationInterface, ManifestationManager};

#[test]
fn resolve_project_root_prefers_prj_root_override() {
    let resolved = resolve_project_root(Some("/tmp/xiuxian-root"), Path::new("/tmp/current"));
    assert_eq!(resolved, PathBuf::from("/tmp/xiuxian-root"));
}

#[test]
fn resolve_prj_data_home_prefers_override_then_defaults() {
    let project_root = Path::new("/tmp/project");

    assert_eq!(
        resolve_prj_data_home(project_root, Some("/tmp/custom-data")),
        PathBuf::from("/tmp/custom-data")
    );
    assert_eq!(
        resolve_prj_data_home(project_root, None),
        PathBuf::from("/tmp/project/.data")
    );
}

#[test]
fn resolve_notebook_root_precedence() {
    let data_home = Path::new("/tmp/project/.data");

    let from_env = resolve_notebook_root(
        data_home,
        Some("/tmp/notebook-env"),
        Some("/tmp/notebook-config"),
    );
    assert_eq!(from_env, PathBuf::from("/tmp/notebook-env"));

    let from_config = resolve_notebook_root(data_home, None, Some("/tmp/notebook-config"));
    assert_eq!(from_config, PathBuf::from("/tmp/notebook-config"));

    let fallback = resolve_notebook_root(data_home, None, None);
    assert_eq!(
        fallback,
        PathBuf::from("/tmp/project/.data/xiuxian/notebook")
    );
}

#[test]
fn resolve_template_globs_prefers_configured_existing_paths() {
    let project_root = std::env::temp_dir().join(format!(
        "xiuxian-template-globs-project-{}",
        std::process::id()
    ));
    let relative_templates = project_root.join("custom/templates");
    let absolute_templates = std::env::temp_dir().join(format!(
        "xiuxian-template-globs-absolute-{}",
        std::process::id()
    ));
    if let Err(error) = fs::create_dir_all(&relative_templates) {
        panic!("create relative templates dir: {error}");
    }
    if let Err(error) = fs::create_dir_all(&absolute_templates) {
        panic!("create absolute templates dir: {error}");
    }

    let globs = resolve_template_globs(
        &project_root,
        Some(vec![
            "custom/templates".to_string(),
            absolute_templates.display().to_string(),
            "   ".to_string(),
        ]),
        None,
    );
    assert_eq!(
        globs,
        vec![
            relative_templates.join("*.md").display().to_string(),
            absolute_templates.join("*.md").display().to_string()
        ]
    );

    let _ = fs::remove_dir_all(&project_root);
    let _ = fs::remove_dir_all(&absolute_templates);
}

#[test]
fn resolve_template_globs_returns_empty_when_no_external_paths_exist() {
    let project_root = Path::new("/tmp/project");
    let globs = resolve_template_globs(project_root, None, None);
    assert!(globs.is_empty());
}

#[test]
fn resolve_template_globs_prefers_xiuxian_resource_root_when_present() {
    let temp_root = std::env::temp_dir().join(format!(
        "xiuxian-resource-root-{}-{}",
        std::process::id(),
        "bootstrap-tests"
    ));
    let template_root = temp_root
        .join("xiuxian-daochang")
        .join("zhixing")
        .join("templates");
    if let Err(error) = fs::create_dir_all(&template_root) {
        panic!("create temp template root: {error}");
    }

    let globs = resolve_template_globs(
        Path::new("/tmp/project"),
        None,
        Some(temp_root.to_string_lossy().as_ref()),
    );
    assert_eq!(
        globs[0],
        template_root.join("*.md").to_string_lossy().into_owned()
    );

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn load_skill_templates_from_embedded_registry_uses_semantic_wendao_uri_links() {
    let manager = ManifestationManager::new_with_embedded_templates(
        &[],
        &[("probe.md", "Skill bridge probe: {{ marker }}")],
    )
    .unwrap_or_else(|error| panic!("create manifestation manager probe: {error}"));
    let summary = load_skill_templates_from_embedded_registry(&manager)
        .unwrap_or_else(|error| panic!("load skill templates from embedded registry: {error}"));
    assert!(summary.linked_ids >= 1);
    assert!(summary.template_records >= 1);
    assert!(summary.loaded_template_names >= 1);

    let rendered = manager
        .render_template("probe.md", json!({ "marker": "ok" }))
        .unwrap_or_else(|error| panic!("render probe template after bridge load: {error}"));
    assert!(rendered.contains("Skill bridge probe: ok"));

    let agenda_rendered = manager
        .render_template(
            "draft_agenda.j2",
            json!({
                "user_request": "Test semantic skill bus loading",
            }),
        )
        .unwrap_or_else(|error| panic!("render semantic linked draft agenda: {error}"));
    assert!(agenda_rendered.contains("<agenda_draft>"));
}

#[test]
fn init_persona_registries_uses_declarative_provider_mode() {
    let config = XiuxianConfig::default();
    let internal_len = init_persona_registries_internal_len(Path::new("/tmp/project"), &config);
    assert_eq!(internal_len, 0);
}
