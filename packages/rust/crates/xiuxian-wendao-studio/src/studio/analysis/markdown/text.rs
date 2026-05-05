use comrak::nodes::{AstNode, NodeValue};

pub(super) fn collect_plain_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut output = String::new();

    for descendant in node.descendants() {
        match &descendant.data().value {
            NodeValue::Text(text) => push_segment(&mut output, text),
            NodeValue::Code(code) => push_segment(&mut output, code.literal.as_str()),
            NodeValue::Math(math) => push_segment(&mut output, math.literal.as_str()),
            NodeValue::WikiLink(link) => push_segment(&mut output, link.url.as_str()),
            NodeValue::LineBreak | NodeValue::SoftBreak => push_segment(&mut output, " "),
            _ => {}
        }
    }

    normalize_whitespace(output.as_str())
}

fn push_segment(output: &mut String, segment: &str) {
    if segment.is_empty() {
        return;
    }
    if !output.is_empty() {
        output.push(' ');
    }
    output.push_str(segment);
}

fn normalize_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}
