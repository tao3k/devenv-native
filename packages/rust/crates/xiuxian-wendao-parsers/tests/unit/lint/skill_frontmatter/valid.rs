use super::{lint_doc, lint_skill};

#[test]
fn lint_accepts_skill_md_with_strict_skill_frontmatter() {
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
        "---\n",
        "# Skill\n",
    ));
    assert!(report.is_clean(), "{report:#?}");
}

#[test]
fn lint_accepts_kind_marked_skill_doc_with_strict_skill_frontmatter() {
    let report = lint_doc(
        "docs/planner.md",
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
