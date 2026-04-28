use anyhow::Result;
use tempfile::TempDir;

mod config_roots;
mod frontmatter;
mod skill_frontmatter;
mod syntax_edges;

pub(super) fn run_lint(temp: &TempDir, scope: Option<&str>) -> Result<(Option<i32>, String)> {
    super::super::run_markdown_lint(temp, scope)
}

pub(super) fn common_doc(title: &str) -> String {
    format!(
        concat!(
            "---\n",
            "kind: reference\n",
            "title: {title}\n",
            "category: docs\n",
            "tags:\n",
            "  - docs\n",
            "description: Demo note\n",
            "author: xiuxian-artisan-workshop\n",
            "date: \"2026-04-26T09:30-07:00\"\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "---\n",
            "# {title}\n",
        ),
        title = title
    )
}

pub(super) fn strict_skill_doc(title: &str, name: &str, source: &str, keyword: &str) -> String {
    format!(
        concat!(
            "---\n",
            "kind: SKILL.md\n",
            "title: {title}\n",
            "category: skills\n",
            "tags:\n",
            "  - {keyword}\n",
            "author: xiuxian-artisan-workshop\n",
            "date: \"2026-04-26T09:30-07:00\"\n",
            "type: skill\n",
            "name: {name}\n",
            "description: Demo skill\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "  author: xiuxian-artisan-workshop\n",
            "  version: \"1.0.0\"\n",
            "  source: \"{source}\"\n",
            "  routing_keywords:\n",
            "    - {keyword}\n",
            "---\n",
            "# {title}\n",
        ),
        title = title,
        name = name,
        source = source,
        keyword = keyword
    )
}
