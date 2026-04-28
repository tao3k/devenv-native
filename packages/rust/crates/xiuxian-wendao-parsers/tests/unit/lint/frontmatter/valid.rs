use xiuxian_wendao_parsers::lint_markdown_syntax;

#[test]
fn lint_accepts_closed_frontmatter_and_fence() {
    let report = lint_markdown_syntax(concat!(
        "---\n",
        "title: demo\n",
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
        "# Heading\n",
        "```rust\n",
        "fn main() {}\n",
        "```\n",
    ));
    assert!(report.is_clean());
}
