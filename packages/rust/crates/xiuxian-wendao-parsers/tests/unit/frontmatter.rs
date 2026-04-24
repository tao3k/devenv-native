use std::path::Path;

use xiuxian_wendao_parsers::frontmatter::{
    discover_skill_documents, frontmatter_kind, is_skill_descriptor_path, parse_frontmatter,
    parse_skill_frontmatter_lenient, skill_frontmatter_has_metadata_mapping,
    skill_frontmatter_name, split_frontmatter, split_frontmatter_raw, uses_skill_frontmatter,
};

#[test]
fn split_frontmatter_returns_yaml_value_and_body() {
    let content = "---\ntitle: Example\n---\n# Body\n";
    let (frontmatter, body) = split_frontmatter(content);
    let title = frontmatter
        .as_ref()
        .and_then(|value| value.get("title"))
        .and_then(serde_yaml::Value::as_str);
    assert_eq!(title, Some("Example"));
    assert_eq!(body, "# Body\n");
}

#[test]
fn split_frontmatter_raw_preserves_yaml_text() {
    let content = "---\nname: planner\nmetadata:\n  version: \"1.0.0\"\n---\n# Body\n";
    let Some(parts) = split_frontmatter_raw(content) else {
        panic!("frontmatter should exist");
    };
    assert_eq!(parts.yaml, "name: planner\nmetadata:\n  version: \"1.0.0\"");
    assert_eq!(parts.body, "# Body\n");
}

#[test]
fn parse_frontmatter_extracts_top_level_fields() {
    let content =
        "---\ntitle: My Note\ndescription: A test\ntags:\n  - python\n  - rust\n---\n# Content";
    let frontmatter = parse_frontmatter(content);
    assert_eq!(frontmatter.title.as_deref(), Some("My Note"));
    assert_eq!(frontmatter.description.as_deref(), Some("A test"));
    assert_eq!(frontmatter.tags, vec!["python", "rust"]);
}

#[test]
fn parse_frontmatter_extracts_skill_metadata() {
    let content = "---\nname: git\ndescription: Git ops\nmetadata:\n  routing_keywords:\n    - commit\n    - branch\n  intents:\n    - version_control\n---\n# SKILL";
    let frontmatter = parse_frontmatter(content);
    assert_eq!(frontmatter.name.as_deref(), Some("git"));
    assert_eq!(frontmatter.routing_keywords, vec!["commit", "branch"]);
    assert_eq!(frontmatter.intents, vec!["version_control"]);
}

#[test]
fn parse_frontmatter_falls_back_to_metadata_tags() {
    let content = "---\nmetadata:\n  tags:\n    - search\n    - vector\n---\n# Content";
    let frontmatter = parse_frontmatter(content);
    assert_eq!(frontmatter.tags, vec!["search", "vector"]);
}

#[test]
fn parse_frontmatter_without_yaml_returns_default() {
    let frontmatter = parse_frontmatter("# No frontmatter");
    assert!(frontmatter.title.is_none());
    assert!(frontmatter.tags.is_empty());
}

#[test]
fn parse_frontmatter_malformed_returns_default() {
    let frontmatter = parse_frontmatter("---\n: bad [[\n---\n");
    assert!(frontmatter.title.is_none());
}

#[test]
fn parse_skill_frontmatter_lenient_accepts_name_without_metadata() {
    let frontmatter = match parse_skill_frontmatter_lenient(
        "---\nname: agenda-management\ndescription: Agenda skill\n---\n# Skill\n",
    ) {
        Ok(Some(frontmatter)) => frontmatter,
        Ok(None) => panic!("frontmatter should exist"),
        Err(error) => panic!("yaml should parse: {error}"),
    };
    assert_eq!(frontmatter.name.as_deref(), Some("agenda-management"));
    assert_eq!(frontmatter.description.as_deref(), Some("Agenda skill"));
}

#[test]
fn parse_skill_frontmatter_lenient_rejects_invalid_yaml() {
    match parse_skill_frontmatter_lenient("---\nname: [oops\n---\n# Skill\n") {
        Ok(_) => panic!("invalid yaml should fail"),
        Err(error) => assert!(!error.to_string().is_empty()),
    }
}

#[test]
fn parse_skill_frontmatter_lenient_returns_none_without_frontmatter() {
    let frontmatter = match parse_skill_frontmatter_lenient("# Skill\n") {
        Ok(frontmatter) => frontmatter,
        Err(error) => panic!("missing frontmatter is allowed: {error}"),
    };
    assert!(frontmatter.is_none());
}

#[test]
fn skill_frontmatter_helpers_follow_parser_owned_contract() {
    let content = concat!(
        "---\n",
        "name: planner\n",
        "metadata:\n",
        "  routing_keywords:\n",
        "    - plan\n",
        "---\n",
        "# Planner\n",
    );
    assert_eq!(skill_frontmatter_name(content).as_deref(), Some("planner"));
    assert!(skill_frontmatter_has_metadata_mapping(content));
}

#[test]
fn uses_skill_frontmatter_accepts_skill_path_or_kind_marker() {
    assert!(uses_skill_frontmatter(
        Some(Path::new("skills/git/SKILL.md")),
        "---\nname: git\nmetadata:\n  version: \"1.0.0\"\n---\n# Skill\n"
    ));
    assert!(uses_skill_frontmatter(
        Some(Path::new("docs/planner.md")),
        "---\nkind: SKILL.md\nname: planner\nmetadata:\n  version: \"1.0.0\"\n---\n# Planner\n"
    ));
    assert_eq!(
        frontmatter_kind(
            "---\nkind: SKILL.md\nname: planner\nmetadata:\n  version: \"1.0.0\"\n---\n# Planner\n"
        )
        .as_deref(),
        Some("SKILL.md")
    );
    assert!(!uses_skill_frontmatter(
        Some(Path::new("docs/note.md")),
        "---\ntitle: Note\n---\n# Note\n"
    ));
}

#[test]
fn discover_skill_documents_returns_deterministic_skill_docs_only() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let root = temp.path();
    std::fs::create_dir_all(root.join("b-skill")).unwrap_or_else(|error| panic!("mkdir: {error}"));
    std::fs::create_dir_all(root.join("a-skill")).unwrap_or_else(|error| panic!("mkdir: {error}"));
    std::fs::create_dir_all(root.join("docs")).unwrap_or_else(|error| panic!("mkdir: {error}"));
    std::fs::write(root.join("b-skill").join("SKILL.md"), "# B\n")
        .unwrap_or_else(|error| panic!("write skill: {error}"));
    std::fs::write(root.join("a-skill").join("skill.md"), "# A\n")
        .unwrap_or_else(|error| panic!("write skill: {error}"));
    std::fs::write(root.join("docs").join("note.md"), "# Note\n")
        .unwrap_or_else(|error| panic!("write note: {error}"));

    let discovered = discover_skill_documents(root);
    let suffixes = discovered
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or_else(|error| panic!("strip prefix: {error}"))
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    assert_eq!(suffixes, vec!["a-skill/skill.md", "b-skill/SKILL.md"]);
    assert!(
        discovered
            .iter()
            .all(|path| is_skill_descriptor_path(Some(path.as_path())))
    );
}
