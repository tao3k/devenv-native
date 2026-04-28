mod fences;
mod frontmatter;
mod kind;
mod skill_frontmatter;
mod wikilinks;

fn lint_with_required_frontmatter(body: &str) -> xiuxian_wendao_parsers::MarkdownSyntaxLintReport {
    let markdown = format!(
        concat!(
            "---\n",
            "title: Demo\n",
            "kind: reference\n",
            "category: docs\n",
            "tags:\n",
            "  - demo\n",
            "description: Demo note\n",
            "author: xiuxian-artisan-workshop\n",
            "date: 2026-04-26T09:30-07:00\n",
            "metadata:\n",
            "  retrieval:\n",
            "    saliency_base: 5.5\n",
            "    decay_rate: 0.05\n",
            "---\n",
            "{}"
        ),
        body
    );
    xiuxian_wendao_parsers::lint_markdown_syntax(markdown.as_str())
}
