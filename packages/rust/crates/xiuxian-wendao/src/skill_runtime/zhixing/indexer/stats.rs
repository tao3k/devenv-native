use crate::parsers::zhixing::tasks::parse_task_projection;

pub(super) fn count_reflection_sections(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("##") && trimmed.to_ascii_lowercase().contains("reflection")
        })
        .count()
}

pub(super) fn count_agenda_statuses(content: &str) -> (usize, usize) {
    content
        .lines()
        .enumerate()
        .fold((0usize, 0usize), |(open, done), (idx, line)| {
            if let Some(task) = parse_task_projection(line, idx + 1) {
                if task.is_completed {
                    (open, done.saturating_add(1))
                } else {
                    (open.saturating_add(1), done)
                }
            } else {
                (open, done)
            }
        })
}

#[cfg(test)]
#[path = "../../../../tests/unit/skill_runtime/zhixing/indexer/stats.rs"]
mod tests;
