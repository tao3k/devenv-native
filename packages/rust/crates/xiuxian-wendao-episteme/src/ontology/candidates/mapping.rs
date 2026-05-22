use std::collections::BTreeSet;

use super::{
    identifiers::mapping_term_candidate_id,
    model::{MappingTerm, MappingTermKind},
};

pub(super) fn extract_mapping_terms(content: &str) -> Vec<MappingTerm> {
    let mut terms = Vec::new();
    let mut heading = String::new();
    let lines = content.lines().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if let Some(title) = heading_title(trimmed) {
            heading = title.to_string();
            index += 1;
            continue;
        }
        if !trimmed.starts_with('|') {
            index += 1;
            continue;
        }
        let start = index;
        while index < lines.len() && lines[index].trim().starts_with('|') {
            index += 1;
        }
        if let Some(kind) = mapping_term_kind_for_heading(heading.as_str()) {
            terms.extend(extract_terms_from_table(&lines[start..index], kind));
        }
    }
    dedupe_mapping_terms(terms)
}

fn extract_terms_from_table(table_lines: &[&str], kind: MappingTermKind) -> Vec<MappingTerm> {
    let rows = table_lines
        .iter()
        .map(|line| parse_org_table_row(line))
        .filter(|cells| !cells.is_empty() && !is_separator_row(cells))
        .collect::<Vec<_>>();
    let Some((header, body)) = rows.split_first() else {
        return Vec::new();
    };
    let stable_key_index = column_index(header, &["stable_key", "稳定键"]);
    let label_index = column_index(header, &["label", "中文主标签"]);
    let note_index = column_index(header, &["note", "说明"]);
    let (Some(stable_key_index), Some(label_index)) = (stable_key_index, label_index) else {
        return Vec::new();
    };
    body.iter()
        .filter_map(|row| {
            let stable_key = row.get(stable_key_index)?.trim();
            let label = row.get(label_index)?.trim();
            if stable_key.is_empty() || label.is_empty() {
                return None;
            }
            let note = note_index
                .and_then(|index| row.get(index))
                .map_or("", String::as_str)
                .trim()
                .to_string();
            Some(MappingTerm {
                candidate_id: mapping_term_candidate_id(stable_key),
                stable_key: stable_key.to_string(),
                label: label.to_string(),
                note,
                term_kind: kind,
            })
        })
        .collect()
}

fn dedupe_mapping_terms(terms: Vec<MappingTerm>) -> Vec<MappingTerm> {
    let mut seen = BTreeSet::new();
    terms
        .into_iter()
        .filter(|term| seen.insert(term.stable_key.clone()))
        .collect()
}

fn heading_title(trimmed: &str) -> Option<&str> {
    let stars = trimmed
        .chars()
        .take_while(|character| *character == '*')
        .count();
    if stars == 0 || !trimmed.chars().nth(stars).is_some_and(char::is_whitespace) {
        return None;
    }
    Some(trimmed[stars..].trim())
}

fn mapping_term_kind_for_heading(heading: &str) -> Option<MappingTermKind> {
    let lower = heading.to_lowercase();
    if heading.contains("对象") || lower.contains("object") {
        Some(MappingTermKind::Object)
    } else if heading.contains("关系") || lower.contains("relation") || lower.contains("link") {
        Some(MappingTermKind::Relation)
    } else {
        None
    }
}

fn parse_org_table_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn is_separator_row(cells: &[String]) -> bool {
    cells.iter().all(|cell| {
        !cell.is_empty()
            && cell
                .chars()
                .all(|character| matches!(character, '-' | '+' | ' '))
    })
}

fn column_index(columns: &[String], names: &[&str]) -> Option<usize> {
    columns.iter().position(|column| {
        let normalized = normalize_column(column);
        names
            .iter()
            .any(|name| normalized == normalize_column(name))
    })
}

fn normalize_column(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
}
