use std::collections::HashMap;

pub(crate) fn render_markdown_skeleton(
    title: &str,
    level: usize,
    attributes: &HashMap<String, String>,
    body: &str,
) -> String {
    let mut lines = Vec::new();
    let heading_level = level.clamp(1, 6);
    lines.push(format!("{} {}", "#".repeat(heading_level), title.trim()));

    if !attributes.is_empty() {
        lines.push(":PROPERTIES:".to_string());
        let mut sorted = attributes.iter().collect::<Vec<_>>();
        sorted.sort_by(|left, right| left.0.cmp(right.0).then_with(|| left.1.cmp(right.1)));
        for (key, value) in sorted {
            lines.push(format!(":{key}: {value}"));
        }
        lines.push(":END:".to_string());
    }

    let mut preserved = body
        .lines()
        .map(str::trim_end)
        .filter(|line| should_preserve_line(line))
        .map(str::to_string)
        .collect::<Vec<_>>();

    if preserved.is_empty()
        && let Some(first_non_empty) = body.lines().map(str::trim).find(|line| !line.is_empty())
    {
        preserved.push(first_non_empty.to_string());
    }

    lines.extend(compact_blank_lines(preserved));
    lines.join("\n").trim().to_string()
}

fn should_preserve_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.starts_with('#')
        || trimmed.starts_with(':')
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
    {
        return true;
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return true;
    }
    let bytes = trimmed.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    index > 0 && bytes.get(index) == Some(&b'.') && bytes.get(index + 1) == Some(&b' ')
}

fn compact_blank_lines(lines: Vec<String>) -> Vec<String> {
    let mut compacted = Vec::with_capacity(lines.len());
    let mut previous_blank = false;
    for line in lines {
        let is_blank = line.trim().is_empty();
        if is_blank && previous_blank {
            continue;
        }
        previous_blank = is_blank;
        compacted.push(line);
    }
    compacted
}
