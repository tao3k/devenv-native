//! SQL validation helpers for dataset-to-ontology materialization.

const FORBIDDEN_SQL_OPERATIONS: &[&str] = &[
    "create", "alter", "drop", "insert", "update", "delete", "merge", "copy", "attach",
];

/// Validate that dataset-to-ontology SQL remains a single read-only query.
///
/// # Errors
///
/// Returns an error when the SQL is empty, contains multiple statements, does
/// not start with `SELECT` or `WITH`, or includes a forbidden mutating keyword.
pub fn validate_dataset_ontology_select_only_sql(sql: &str) -> Result<(), String> {
    let sanitized = strip_sql_comments_and_literals(sql);
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        return Err("dataset ontology SQL must not be empty".to_string());
    }
    let query = trimmed.strip_suffix(';').map_or(trimmed, str::trim_end);
    if query.contains(';') {
        return Err("dataset ontology SQL must contain exactly one statement".to_string());
    }

    let first_token = query
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .find(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "dataset ontology SQL must contain a query".to_string())?;
    if first_token != "select" && first_token != "with" {
        return Err("dataset ontology SQL must start with SELECT or WITH".to_string());
    }

    let forbidden = sanitized
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .find(|token| FORBIDDEN_SQL_OPERATIONS.contains(&token.as_str()));
    if let Some(token) = forbidden {
        return Err(format!(
            "dataset ontology SQL contains forbidden operation `{token}`"
        ));
    }
    Ok(())
}

fn strip_sql_comments_and_literals(sql: &str) -> String {
    let without_line_comments = strip_line_comments(sql);
    let without_block_comments = strip_block_comments(&without_line_comments);
    strip_single_quoted_literals(&without_block_comments)
}

fn strip_line_comments(sql: &str) -> String {
    sql.lines()
        .map(|line| line.split_once("--").map_or(line, |(before, _)| before))
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_block_comments(sql: &str) -> String {
    let mut output = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    let mut in_comment = false;
    while let Some(character) = chars.next() {
        if in_comment {
            if character == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_comment = false;
            }
            output.push(' ');
            continue;
        }
        if character == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_comment = true;
            output.push(' ');
            continue;
        }
        output.push(character);
    }
    output
}

fn strip_single_quoted_literals(sql: &str) -> String {
    let mut output = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    let mut in_literal = false;
    while let Some(character) = chars.next() {
        if in_literal {
            if character == '\'' {
                if chars.peek() == Some(&'\'') {
                    let _ = chars.next();
                } else {
                    in_literal = false;
                }
            }
            output.push(' ');
            continue;
        }
        if character == '\'' {
            in_literal = true;
            output.push(' ');
            continue;
        }
        output.push(character);
    }
    output
}
