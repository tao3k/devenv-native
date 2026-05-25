//! Compact Org section features for recovery and recall.

#[derive(Debug, Clone, Default)]
pub(super) struct TaskSectionLens {
    pub(super) checkboxes: Vec<String>,
    pub(super) direct_children: Vec<String>,
    pub(super) next_unchecked: Option<String>,
    pub(super) checked: usize,
    pub(super) unchecked: usize,
}

impl TaskSectionLens {
    pub(super) fn from_section(section: &str) -> Self {
        let root_level = section_heading_level(section).unwrap_or(1);
        let mut all_checkboxes = Vec::new();
        let mut root_checkboxes = Vec::new();
        let mut child_checklists = Vec::<(String, Vec<String>)>::new();
        let mut current_child_index = None;
        let mut direct_children = Vec::new();

        for line in section.lines().skip(1) {
            let trimmed = line.trim_start();
            if let Some(level) = heading_level(trimmed)
                && level == root_level + 1
            {
                direct_children.push(trimmed.to_string());
                current_child_index = Some(child_checklists.len());
                child_checklists.push((trimmed.to_string(), Vec::new()));
            } else if let Some(level) = heading_level(trimmed)
                && level <= root_level
            {
                current_child_index = None;
            }
            if is_checkbox_line(trimmed) {
                let checkbox = trimmed.to_string();
                all_checkboxes.push(checkbox.clone());
                if let Some(index) = current_child_index {
                    child_checklists[index].1.push(checkbox);
                } else {
                    root_checkboxes.push(checkbox);
                }
            }
        }

        let checkboxes = primary_checkboxes(child_checklists, root_checkboxes, all_checkboxes);
        let next_unchecked = checkboxes
            .iter()
            .find(|line| is_unchecked_checkbox_line(line))
            .cloned();
        let checked = checkboxes
            .iter()
            .filter(|line| !is_unchecked_checkbox_line(line))
            .count();
        let unchecked = checkboxes.len().saturating_sub(checked);

        Self {
            checkboxes,
            direct_children,
            next_unchecked,
            checked,
            unchecked,
        }
    }

    pub(super) fn progress_label(&self) -> Option<String> {
        let total = self.checked + self.unchecked;
        if total == 0 {
            return None;
        }
        let percent = self.checked * 100 / total;
        Some(format!("[{}/{}] [{}%]", self.checked, total, percent))
    }

    pub(super) fn is_complete(&self) -> bool {
        self.checked > 0 && self.unchecked == 0
    }

    pub(super) fn section_has_reflection_content(section: &str) -> bool {
        section_has_direct_child_content(section, "reflection")
    }

    pub(super) fn checklist_text(&self) -> String {
        self.checkboxes
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(super) fn child_heading_text(&self) -> String {
        self.direct_children
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn primary_checkboxes(
    child_checklists: Vec<(String, Vec<String>)>,
    root_checkboxes: Vec<String>,
    all_checkboxes: Vec<String>,
) -> Vec<String> {
    child_checklists
        .into_iter()
        .find(|(heading, checkboxes)| {
            heading.to_lowercase().contains("checklist") && !checkboxes.is_empty()
        })
        .map(|(_, checkboxes)| checkboxes)
        .filter(|checkboxes| !checkboxes.is_empty())
        .or_else(|| (!root_checkboxes.is_empty()).then_some(root_checkboxes))
        .unwrap_or(all_checkboxes)
}

fn section_heading_level(section: &str) -> Option<usize> {
    section
        .lines()
        .next()
        .and_then(|line| heading_level(line.trim_start()))
}

fn heading_level(line: &str) -> Option<usize> {
    let level = line
        .chars()
        .take_while(|character| *character == '*')
        .count();
    (level > 0 && line.as_bytes().get(level) == Some(&b' ')).then_some(level)
}

fn section_has_direct_child_content(section: &str, title: &str) -> bool {
    let Some(root_level) = section_heading_level(section) else {
        return false;
    };
    let target_level = root_level + 1;
    let mut in_target = false;
    let mut in_drawer = false;

    for line in section.lines().skip(1) {
        let trimmed = line.trim_start();
        if let Some(level) = heading_level(trimmed) {
            if in_target && level <= target_level {
                break;
            }
            in_target = level == target_level
                && heading_title(trimmed)
                    .is_some_and(|heading| heading.eq_ignore_ascii_case(title));
            in_drawer = false;
            continue;
        }

        if in_target && reflection_line_has_content(trimmed, &mut in_drawer) {
            return true;
        }
    }

    false
}

fn heading_title(line: &str) -> Option<&str> {
    let level = heading_level(line)?;
    let mut title = line[level..].trim();
    if let Some(rest) = title
        .split_once(char::is_whitespace)
        .and_then(|(head, rest)| agent_task_todo_keyword(head).then_some(rest.trim()))
    {
        title = rest;
    }
    if title.starts_with("[#") && title.get(3..4) == Some("]") {
        title = title[4..].trim_start();
    }
    Some(strip_heading_tags(title).trim())
}

fn agent_task_todo_keyword(value: &str) -> bool {
    matches!(
        value,
        "TODO" | "DOING" | "NEXT" | "WAITING" | "DONE" | "CANCELLED"
    )
}

fn strip_heading_tags(title: &str) -> &str {
    let Some((before, after)) = title.rsplit_once(' ') else {
        return title;
    };
    if after.starts_with(':')
        && after.ends_with(':')
        && after.trim_matches(':').split(':').all(|tag| {
            !tag.is_empty()
                && tag
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '#')
        })
    {
        before
    } else {
        title
    }
}

fn reflection_line_has_content(line: &str, in_drawer: &mut bool) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }
    if *in_drawer {
        if trimmed.eq_ignore_ascii_case(":END:") {
            *in_drawer = false;
        }
        return false;
    }
    if trimmed.eq_ignore_ascii_case(":PROPERTIES:") || org_drawer_start(trimmed) {
        *in_drawer = true;
        return false;
    }
    !reflection_placeholder_line(trimmed)
}

fn org_drawer_start(trimmed: &str) -> bool {
    trimmed.len() > 2
        && trimmed.starts_with(':')
        && trimmed.ends_with(':')
        && trimmed[1..trimmed.len() - 1]
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch == '_' || ch.is_ascii_digit())
}

fn reflection_placeholder_line(trimmed: &str) -> bool {
    matches!(
        trimmed,
        "- Summary:"
            | "- Scope drift or decision notes:"
            | "- Validation evidence:"
            | "- Documentation sync:"
            | "- Durable follow-up:"
            | "Summary:"
            | "Scope drift or decision notes:"
            | "Validation evidence:"
            | "Documentation sync:"
            | "Durable follow-up:"
    )
}

fn is_checkbox_line(line: &str) -> bool {
    line.starts_with("- [ ]")
        || line.starts_with("- [X]")
        || line.starts_with("- [x]")
        || line.starts_with("+ [ ]")
        || line.starts_with("+ [X]")
        || line.starts_with("+ [x]")
}

fn is_unchecked_checkbox_line(line: &str) -> bool {
    line.starts_with("- [ ]") || line.starts_with("+ [ ]")
}
