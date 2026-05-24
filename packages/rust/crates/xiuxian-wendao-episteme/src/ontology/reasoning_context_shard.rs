//! Deterministic reasoning-context sharding for LLM review tasks.

use std::fmt::Write as _;

use anyhow::{Result, bail};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) const REASONING_CONTEXT_SHARD_MODE_DISABLED: &str = "disabled";
pub(crate) const REASONING_CONTEXT_SHARD_MODE_SERVICE_CATALOG_TABLE_ROWS: &str =
    "service-catalog-table-rows";
pub(crate) const SHARD_KIND_SERVICE_CATALOG_TABLE_ROWS: &str = "service_catalog_table_rows";

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct EpistemeReasoningContextShardSource<'a> {
    pub subject_id: &'a str,
    pub context_id: &'a str,
    pub target_field_group: &'a str,
    pub service_catalog_field_group: &'a str,
    pub extracted_text: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeReasoningContextShard {
    pub shard_id: String,
    pub parent_subject_id: String,
    pub shard_index: usize,
    pub shard_count: usize,
    pub shard_kind: &'static str,
    pub table_index: usize,
    pub row_start: usize,
    pub row_end: usize,
    pub row_count: usize,
    pub carry_forward_first_column: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct EpistemeReasoningContextShardText {
    pub shard: Option<EpistemeReasoningContextShard>,
    pub extracted_text: String,
    pub text_sha256: String,
    pub text_char_count: usize,
}

pub(crate) fn validate_reasoning_context_shard_mode(mode: &str) -> Result<()> {
    match mode {
        REASONING_CONTEXT_SHARD_MODE_DISABLED
        | REASONING_CONTEXT_SHARD_MODE_SERVICE_CATALOG_TABLE_ROWS => Ok(()),
        other => bail!(
            "unsupported reasoning context shard mode `{other}`; expected `disabled` or `service-catalog-table-rows`"
        ),
    }
}

pub(crate) fn plan_episteme_reasoning_context_shard_texts(
    source: &EpistemeReasoningContextShardSource<'_>,
    mode: &str,
    row_limit: usize,
) -> Result<Vec<EpistemeReasoningContextShardText>> {
    validate_reasoning_context_shard_mode(mode)?;
    if row_limit == 0 {
        bail!("reasoning context shard row limit must be greater than zero");
    }
    if mode == REASONING_CONTEXT_SHARD_MODE_DISABLED
        || source.target_field_group != source.service_catalog_field_group
    {
        return Ok(vec![EpistemeReasoningContextShardText {
            shard: None,
            extracted_text: source.extracted_text.to_owned(),
            text_sha256: sha256_text(source.extracted_text),
            text_char_count: source.extracted_text.chars().count(),
        }]);
    }
    service_catalog_table_row_shards(source, row_limit)
}

fn service_catalog_table_row_shards(
    source: &EpistemeReasoningContextShardSource<'_>,
    row_limit: usize,
) -> Result<Vec<EpistemeReasoningContextShardText>> {
    let Some(table) = first_markdown_table(source.extracted_text) else {
        bail!(
            "subject `{}` requested service-catalog reasoning context sharding but context `{}` has no Markdown table",
            source.subject_id,
            source.context_id
        );
    };
    if table.data_rows.is_empty() {
        bail!(
            "subject `{}` requested service-catalog reasoning context sharding but context `{}` has no table data rows",
            source.subject_id,
            source.context_id
        );
    }

    let chunk_count = table.data_rows.len().div_ceil(row_limit);
    let mut planned = Vec::with_capacity(chunk_count);
    for chunk_index in 0..chunk_count {
        let start = chunk_index * row_limit;
        let end = (start + row_limit).min(table.data_rows.len());
        let row_start = start + 1;
        let row_end = end;
        let shard = EpistemeReasoningContextShard {
            shard_id: stable_shard_id(source, row_start, row_end),
            parent_subject_id: source.subject_id.to_owned(),
            shard_index: chunk_index + 1,
            shard_count: chunk_count,
            shard_kind: SHARD_KIND_SERVICE_CATALOG_TABLE_ROWS,
            table_index: 1,
            row_start,
            row_end,
            row_count: end - start,
            carry_forward_first_column: carry_forward_first_column(&table, start),
        };
        let extracted_text = sharded_table_text(&table, &shard, start, end);
        planned.push(EpistemeReasoningContextShardText {
            text_sha256: sha256_text(&extracted_text),
            text_char_count: extracted_text.chars().count(),
            extracted_text,
            shard: Some(shard),
        });
    }
    Ok(planned)
}

fn sharded_table_text(
    table: &MarkdownTable,
    shard: &EpistemeReasoningContextShard,
    start: usize,
    end: usize,
) -> String {
    let mut extracted_text = String::new();
    let _ = writeln!(
        extracted_text,
        "Reasoning context shard: {} {}/{}; review only table data rows {}-{}.",
        shard.shard_kind, shard.shard_index, shard.shard_count, shard.row_start, shard.row_end
    );
    if let Some(carry_forward) = &shard.carry_forward_first_column {
        let _ = writeln!(
            extracted_text,
            "Carry-forward first-column value for blank cells: {carry_forward}."
        );
    }
    extracted_text.push_str(&table.header);
    extracted_text.push('\n');
    extracted_text.push_str(&table.separator);
    extracted_text.push('\n');
    for data_row in &table.data_rows[start..end] {
        extracted_text.push_str(data_row);
        extracted_text.push('\n');
    }
    extracted_text
}

struct MarkdownTable {
    header: String,
    separator: String,
    data_rows: Vec<String>,
}

fn first_markdown_table(text: &str) -> Option<MarkdownTable> {
    let mut table_lines = Vec::<String>::new();
    for line in text.lines() {
        if is_table_line(line) {
            table_lines.push(line.trim().to_owned());
        } else if table_lines.len() >= 3 {
            break;
        } else {
            table_lines.clear();
        }
    }
    if table_lines.len() < 3 || !is_separator_row(table_lines.get(1)?) {
        return None;
    }
    Some(MarkdownTable {
        header: table_lines[0].clone(),
        separator: table_lines[1].clone(),
        data_rows: table_lines[2..].to_vec(),
    })
}

fn is_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.matches('|').count() >= 3
}

fn is_separator_row(line: &str) -> bool {
    table_cells(line).iter().all(|cell| {
        let cell = cell.trim();
        !cell.is_empty()
            && cell.chars().all(|ch| matches!(ch, '-' | ':' | ' ' | '\t'))
            && cell.contains('-')
    })
}

fn carry_forward_first_column(table: &MarkdownTable, start: usize) -> Option<String> {
    let current = table
        .data_rows
        .get(start)
        .and_then(|row| table_cells(row).first().cloned())
        .unwrap_or_default();
    if !current.trim().is_empty() {
        return None;
    }
    table.data_rows[..start].iter().rev().find_map(|row| {
        let value = table_cells(row).first().cloned().unwrap_or_default();
        (!value.trim().is_empty()).then(|| value.trim().to_owned())
    })
}

fn table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect()
}

fn stable_shard_id(
    source: &EpistemeReasoningContextShardSource<'_>,
    row_start: usize,
    row_end: usize,
) -> String {
    let digest = Sha256::digest(
        format!(
            "{}:{}:{row_start}:{row_end}",
            source.subject_id, source.context_id
        )
        .as_bytes(),
    );
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("structural_facts.reasoning_context_shard.{suffix}")
}

fn sha256_text(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
