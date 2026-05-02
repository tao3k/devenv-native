use super::{GraphqlQueryArgs, QueryCommand, RestQueryArgs, SqlQueryArgs, query};
use crate::bin_support::wendao::types::Command;

#[test]
fn test_graphql_query_args_capture_document_text() {
    let args = GraphqlQueryArgs {
        document: "{ wendao_sql_tables { sql_table_name } }".to_string(),
    };
    assert_eq!(args.document, "{ wendao_sql_tables { sql_table_name } }");
}

#[test]
fn test_rest_query_args_capture_payload_text() {
    let args = RestQueryArgs {
        payload: r#"{"query_language":"sql","query":"SELECT 1"}"#.to_string(),
    };
    assert_eq!(
        args.payload,
        r#"{"query_language":"sql","query":"SELECT 1"}"#
    );
}

#[test]
fn test_sql_query_args_capture_query_text() {
    let args = SqlQueryArgs {
        query: "SELECT * FROM reference_occurrence".to_string(),
    };
    assert_eq!(args.query, "SELECT * FROM reference_occurrence");
}

#[test]
fn test_graphql_query_command_creation() {
    let command = query(QueryCommand::Graphql(GraphqlQueryArgs {
        document: "{ reference_occurrence { name } }".to_string(),
    }));
    match command {
        Command::Query { command } => match command {
            QueryCommand::Graphql(args) => {
                assert_eq!(args.document, "{ reference_occurrence { name } }");
            }
            QueryCommand::Rest(_) | QueryCommand::Sql(_) => {
                panic!("expected GraphQL query command");
            }
        },
        other => panic!("expected query command, got {other:?}"),
    }
}

#[test]
fn test_rest_query_command_creation() {
    let command = query(QueryCommand::Rest(RestQueryArgs {
        payload: r#"{"query_language":"graphql","document":"{ reference_occurrence { name } }"}"#
            .to_string(),
    }));
    match command {
        Command::Query { command } => match command {
            QueryCommand::Rest(args) => assert_eq!(
                args.payload,
                r#"{"query_language":"graphql","document":"{ reference_occurrence { name } }"}"#
            ),
            QueryCommand::Graphql(_) | QueryCommand::Sql(_) => {
                panic!("expected REST query command");
            }
        },
        other => panic!("expected query command, got {other:?}"),
    }
}

#[test]
fn test_query_command_creation() {
    let command = query(QueryCommand::Sql(SqlQueryArgs {
        query: "SELECT name FROM local_symbol".to_string(),
    }));
    match command {
        Command::Query { command } => match command {
            QueryCommand::Graphql(_) | QueryCommand::Rest(_) => {
                panic!("expected SQL query command");
            }
            QueryCommand::Sql(args) => assert_eq!(args.query, "SELECT name FROM local_symbol"),
        },
        other => panic!("expected query command, got {other:?}"),
    }
}
