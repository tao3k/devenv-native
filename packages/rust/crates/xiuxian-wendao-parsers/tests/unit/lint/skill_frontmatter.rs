use std::path::Path;

use xiuxian_wendao_parsers::{MarkdownSyntaxLintCode, lint_markdown_syntax_with_path};

#[test]
fn lint_reports_missing_skill_frontmatter_name() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "description: Demo skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords:\n",
            "    - demo\n",
            "---\n",
            "# Skill\n",
        ),
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::MissingSkillFrontmatterName
    );
}

#[test]
fn lint_reports_missing_skill_frontmatter_metadata() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "name: demo-skill\n",
            "description: Demo skill\n",
            "---\n",
            "# Skill\n",
        ),
    );
    let codes = report
        .issues
        .iter()
        .map(|issue| issue.code)
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&MarkdownSyntaxLintCode::MissingFrontmatterRetrievalSaliencyBase),
        "{report:#?}"
    );
    assert!(
        codes.contains(&MarkdownSyntaxLintCode::MissingFrontmatterRetrievalDecayRate),
        "{report:#?}"
    );
    assert!(
        codes.contains(&MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata),
        "{report:#?}"
    );
}

#[test]
fn lint_reports_skill_frontmatter_missing_metadata_version() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "name: demo-skill\n",
            "description: Demo skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords:\n",
            "    - demo\n",
            "---\n",
            "# Skill\n",
        ),
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema
    );
    assert_eq!(
        report.issues[0].message,
        "skill frontmatter `metadata.version` must be a non-empty string"
    );
    assert_eq!((report.issues[0].line, report.issues[0].column), (12, 1));
}

#[test]
fn lint_reports_skill_frontmatter_missing_required_schema_fields() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "description: Demo skill\n",
            "name: demo-skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  version: \"1.0.0\"\n",
            "---\n",
            "# Skill\n",
        ),
    );
    let messages = report
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(report.issues.len(), 3);
    assert!(
        report
            .issues
            .iter()
            .all(|issue| { issue.code == MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema })
    );
    assert!(messages.contains(&"skill frontmatter top-level `type` must be `skill`"));
    assert!(messages.contains(&"skill frontmatter `metadata.source` must be a non-empty string"));
    assert!(messages.contains(
        &"skill frontmatter `metadata.routing_keywords` must be a non-empty string array"
    ));
}

#[test]
fn lint_reports_skill_frontmatter_metadata_sequence_schema() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "name: demo-skill\n",
            "description: Demo skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords: demo\n",
            "  intents:\n",
            "    - Use the demo skill\n",
            "---\n",
            "# Skill\n",
        ),
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema
    );
    assert_eq!(
        report.issues[0].message,
        "skill frontmatter `metadata.routing_keywords` must be a non-empty string array"
    );
    assert_eq!((report.issues[0].line, report.issues[0].column), (19, 3));
}

#[test]
fn lint_reports_skill_frontmatter_optional_intents_schema() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "name: demo-skill\n",
            "description: Demo skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords:\n",
            "    - demo\n",
            "  intents: Use the demo skill\n",
            "---\n",
            "# Skill\n",
        ),
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema
    );
    assert_eq!(
        report.issues[0].message,
        "skill frontmatter `metadata.intents` must be a non-empty string array"
    );
    assert_eq!((report.issues[0].line, report.issues[0].column), (21, 3));
}

#[test]
fn lint_reports_skill_frontmatter_legacy_keywords() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "name: demo-skill\n",
            "description: Demo skill\n",
            "keywords:\n",
            "  - demo\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords:\n",
            "    - demo\n",
            "---\n",
            "# Skill\n",
        ),
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].code,
        MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema
    );
    assert_eq!(
        report.issues[0].message,
        "skill frontmatter must use `metadata.routing_keywords`; legacy top-level `keywords` is not allowed"
    );
    assert_eq!((report.issues[0].line, report.issues[0].column), (12, 1));
}

#[test]
fn lint_accepts_skill_md_with_strict_skill_frontmatter() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("skills/demo/SKILL.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Demo Skill\n",
            "category: skills\n",
            "tags:\n",
            "  - demo\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "name: demo-skill\n",
            "description: Demo skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/demo\"\n",
            "  routing_keywords:\n",
            "    - demo\n",
            "---\n",
            "# Skill\n",
        ),
    );
    assert!(report.is_clean(), "{report:#?}");
}

#[test]
fn lint_accepts_kind_marked_skill_doc_with_strict_skill_frontmatter() {
    let report = lint_markdown_syntax_with_path(
        Some(Path::new("docs/planner.md")),
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: Planner\n",
            "category: skills\n",
            "tags:\n",
            "  - planner\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "type: skill\n",
            "name: planner\n",
            "description: Planning skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"https://example.test/skills/planner\"\n",
            "  routing_keywords:\n",
            "    - planner\n",
            "  intents:\n",
            "    - Use the planner skill\n",
            "---\n",
            "# Planner\n",
        ),
    );
    assert!(report.is_clean(), "{report:#?}");
}
