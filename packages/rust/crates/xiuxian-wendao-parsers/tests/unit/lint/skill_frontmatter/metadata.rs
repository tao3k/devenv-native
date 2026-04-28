use xiuxian_wendao_parsers::MarkdownSyntaxLintCode;

use super::lint_skill;

#[test]
fn lint_reports_skill_frontmatter_metadata_sequence_schema() {
    let report = lint_skill(concat!(
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
    ));
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
    let report = lint_skill(concat!(
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
    ));
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
    let report = lint_skill(concat!(
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
    ));
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
