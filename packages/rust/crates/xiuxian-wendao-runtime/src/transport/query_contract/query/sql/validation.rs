//! SQL query metadata validation for Wendao Flight routes.

use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

/// Validate the stable read-only SQL request contract.
///
/// # Errors
///
/// Returns an error when the query text is blank, parses as multiple
/// statements, or resolves to anything other than one read-only `SELECT`-style
/// query statement.
pub fn validate_sql_query_request(query_text: &str) -> Result<(), String> {
    let normalized_query = query_text.trim();
    if normalized_query.is_empty() {
        return Err("SQL query text must not be blank".to_string());
    }

    let dialect = GenericDialect {};
    let mut statements = Parser::parse_sql(&dialect, normalized_query)
        .map_err(|error| format!("failed to parse SQL query text: {error}"))?;
    if statements.len() != 1 {
        return Err("SQL query text must contain exactly one statement".to_string());
    }

    let statement = statements
        .pop()
        .ok_or_else(|| "SQL query text must contain exactly one statement".to_string())?;
    match statement {
        Statement::Query(_) => Ok(()),
        _ => Err("SQL query text must be a read-only query statement".to_string()),
    }
}
