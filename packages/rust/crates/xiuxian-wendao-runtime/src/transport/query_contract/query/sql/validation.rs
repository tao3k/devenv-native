//! SQL query metadata validation for Wendao Flight routes.

use datafusion::sql::parser::{DFParser, Statement as DataFusionStatement};
use datafusion::sql::sqlparser::ast::Statement as SqlStatement;

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

    let mut statements = DFParser::parse_sql(normalized_query)
        .map_err(|error| format!("failed to parse SQL query text: {error}"))?;
    if statements.len() != 1 {
        return Err("SQL query text must contain exactly one statement".to_string());
    }

    let statement = statements
        .pop_front()
        .ok_or_else(|| "SQL query text must contain exactly one statement".to_string())?;
    match statement {
        DataFusionStatement::Statement(statement)
            if matches!(statement.as_ref(), SqlStatement::Query(_)) =>
        {
            Ok(())
        }
        _ => Err("SQL query text must be a read-only query statement".to_string()),
    }
}
