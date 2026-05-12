use graphql_parser::query as gql;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StudioGraphqlTableQuery {
    pub(crate) response_key: String,
    pub(crate) table_name: String,
    pub(crate) columns: Vec<String>,
    pub(crate) filters: Vec<StudioGraphqlFilterPredicate>,
    pub(crate) sort: Vec<StudioGraphqlSortOption>,
    pub(crate) limit: Option<usize>,
    pub(crate) page: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StudioGraphqlFilterPredicate {
    pub(crate) column_name: String,
    pub(crate) operator: StudioGraphqlFilterOperator,
    pub(crate) value: StudioGraphqlScalarValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StudioGraphqlFilterOperator {
    Eq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StudioGraphqlScalarValue {
    Boolean(bool),
    String(String),
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StudioGraphqlSortOption {
    pub(crate) field_name: String,
    pub(crate) descending: bool,
}

pub(crate) fn parse_graphql_document(document: &str) -> Result<StudioGraphqlTableQuery, String> {
    let parsed = gql::parse_query::<String>(document)
        .map_err(|error| format!("failed to parse GraphQL document: {error}"))?;
    let selection_set = root_selection_set(&parsed)?;
    if selection_set.items.len() != 1 {
        return Err("GraphQL document must contain exactly one root field".to_string());
    }
    parse_root_selection(&selection_set.items[0])
}

fn root_selection_set<'a>(
    document: &'a gql::Document<'a, String>,
) -> Result<&'a gql::SelectionSet<'a, String>, String> {
    let Some(definition) = document.definitions.first() else {
        return Err("GraphQL document must contain one query operation".to_string());
    };

    match definition {
        gql::Definition::Operation(gql::OperationDefinition::SelectionSet(selection_set)) => {
            Ok(selection_set)
        }
        gql::Definition::Operation(gql::OperationDefinition::Query(query)) => {
            Ok(&query.selection_set)
        }
        gql::Definition::Operation(_) => {
            Err("only GraphQL query operations are supported".to_string())
        }
        gql::Definition::Fragment(_) => {
            Err("GraphQL fragments are not supported in the first adapter slice".to_string())
        }
    }
}

fn parse_root_selection<'a>(
    selection: &'a gql::Selection<'a, String>,
) -> Result<StudioGraphqlTableQuery, String> {
    let gql::Selection::Field(field) = selection else {
        return Err("only root GraphQL fields are supported".to_string());
    };
    parse_table_query(field)
}

fn parse_table_query(field: &gql::Field<'_, String>) -> Result<StudioGraphqlTableQuery, String> {
    ensure_known_arguments(field, &["filter", "sort", "limit", "page"])?;
    let columns = parse_projection_columns(&field.selection_set)?;
    let filters = parse_filters(field)?;
    let sort = parse_sort(field)?;
    let limit = optional_usize_argument(field, "limit")?;
    let page = optional_usize_argument(field, "page")?;

    if page.is_some() && limit.is_none() {
        return Err("GraphQL field `page` requires `limit`".to_string());
    }

    Ok(StudioGraphqlTableQuery {
        response_key: response_key(field),
        table_name: field.name.clone(),
        columns,
        filters,
        sort,
        limit,
        page,
    })
}

fn parse_projection_columns(
    selection_set: &gql::SelectionSet<'_, String>,
) -> Result<Vec<String>, String> {
    let columns = selection_set
        .items
        .iter()
        .map(|selection| {
            let gql::Selection::Field(field) = selection else {
                return Err("GraphQL table queries only support field selections".to_string());
            };
            ensure_empty_selection(field)?;
            Ok(field.name.clone())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if columns.is_empty() {
        return Err("GraphQL table queries must request at least one field".to_string());
    }
    Ok(columns)
}

fn parse_filters(
    field: &gql::Field<'_, String>,
) -> Result<Vec<StudioGraphqlFilterPredicate>, String> {
    let Some(argument) = argument_value(field, "filter") else {
        return Ok(Vec::new());
    };
    let gql::Value::Object(filters) = argument else {
        return Err(format!(
            "GraphQL field `{}` expects object argument `filter`",
            field.name
        ));
    };

    let mut predicates = Vec::new();
    for (column_name, filter) in filters {
        match filter {
            gql::Value::Object(predicate_object) => {
                for (operator_name, operand) in predicate_object {
                    predicates.push(StudioGraphqlFilterPredicate {
                        column_name: column_name.clone(),
                        operator: parse_filter_operator(field, operator_name)?,
                        value: parse_scalar_value(field, operand, "filter")?,
                    });
                }
            }
            gql::Value::Boolean(_)
            | gql::Value::String(_)
            | gql::Value::Int(_)
            | gql::Value::Float(_) => predicates.push(StudioGraphqlFilterPredicate {
                column_name: column_name.clone(),
                operator: StudioGraphqlFilterOperator::Eq,
                value: parse_scalar_value(field, filter, "filter")?,
            }),
            other => {
                return Err(format!(
                    "GraphQL field `{}` received unsupported filter operand `{other}`",
                    field.name
                ));
            }
        }
    }

    Ok(predicates)
}

fn parse_sort(field: &gql::Field<'_, String>) -> Result<Vec<StudioGraphqlSortOption>, String> {
    let Some(argument) = argument_value(field, "sort") else {
        return Ok(Vec::new());
    };
    let gql::Value::List(sort_entries) = argument else {
        return Err(format!(
            "GraphQL field `{}` expects list argument `sort`",
            field.name
        ));
    };

    let mut sort = Vec::new();
    for entry in sort_entries {
        let gql::Value::Object(options) = entry else {
            return Err(format!(
                "GraphQL field `{}` sort entries must be objects",
                field.name
            ));
        };

        let field_name = match options.get("field") {
            Some(gql::Value::String(field_name)) => field_name.clone(),
            Some(other) => {
                return Err(format!(
                    "GraphQL field `{}` sort option `field` must be a string, got `{other}`",
                    field.name
                ));
            }
            None => {
                return Err(format!(
                    "GraphQL field `{}` sort options require `field`",
                    field.name
                ));
            }
        };

        let descending = match options.get("order") {
            None => false,
            Some(gql::Value::String(order)) => match order.as_str() {
                "asc" => false,
                "desc" => true,
                other => {
                    return Err(format!(
                        "GraphQL field `{}` received unsupported sort order `{other}`",
                        field.name
                    ));
                }
            },
            Some(other) => {
                return Err(format!(
                    "GraphQL field `{}` sort option `order` must be a string, got `{other}`",
                    field.name
                ));
            }
        };

        sort.push(StudioGraphqlSortOption {
            field_name,
            descending,
        });
    }

    Ok(sort)
}

fn response_key(field: &gql::Field<'_, String>) -> String {
    field.alias.clone().unwrap_or_else(|| field.name.clone())
}

fn ensure_known_arguments(
    field: &gql::Field<'_, String>,
    known_arguments: &[&str],
) -> Result<(), String> {
    for (argument_name, _) in &field.arguments {
        if !known_arguments.iter().any(|known| known == argument_name) {
            return Err(format!(
                "GraphQL field `{}` received unsupported argument `{argument_name}`",
                field.name
            ));
        }
    }
    Ok(())
}

fn ensure_empty_selection(field: &gql::Field<'_, String>) -> Result<(), String> {
    if field.selection_set.items.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "GraphQL leaf field `{}` must not contain a nested selection set",
            field.name
        ))
    }
}

fn argument_value<'a, 'b>(
    field: &'a gql::Field<'b, String>,
    name: &str,
) -> Option<&'a gql::Value<'b, String>> {
    field
        .arguments
        .iter()
        .find_map(|(key, value)| (key == name).then_some(value))
}

fn optional_usize_argument(
    field: &gql::Field<'_, String>,
    name: &str,
) -> Result<Option<usize>, String> {
    let Some(value) = argument_value(field, name) else {
        return Ok(None);
    };

    match value {
        gql::Value::Int(number) => Ok(Some(
            usize::try_from(number.as_i64().ok_or_else(|| {
                format!(
                    "GraphQL field `{}` integer argument `{name}` is out of range",
                    field.name
                )
            })?)
            .map_err(|_| {
                format!(
                    "GraphQL field `{}` integer argument `{name}` is out of range",
                    field.name
                )
            })?,
        )),
        other => Err(format!(
            "GraphQL field `{}` requires integer argument `{name}`, got `{other}`",
            field.name
        )),
    }
}

fn parse_filter_operator(
    field: &gql::Field<'_, String>,
    operator_name: &str,
) -> Result<StudioGraphqlFilterOperator, String> {
    match operator_name {
        "eq" => Ok(StudioGraphqlFilterOperator::Eq),
        "lt" => Ok(StudioGraphqlFilterOperator::Lt),
        "lte" | "lteq" => Ok(StudioGraphqlFilterOperator::LtEq),
        "gt" => Ok(StudioGraphqlFilterOperator::Gt),
        "gte" | "gteq" => Ok(StudioGraphqlFilterOperator::GtEq),
        other => Err(format!(
            "GraphQL field `{}` received unsupported filter operator `{other}`",
            field.name
        )),
    }
}

fn parse_scalar_value(
    field: &gql::Field<'_, String>,
    value: &gql::Value<'_, String>,
    argument_name: &str,
) -> Result<StudioGraphqlScalarValue, String> {
    match value {
        gql::Value::Boolean(value) => Ok(StudioGraphqlScalarValue::Boolean(*value)),
        gql::Value::String(value) => Ok(StudioGraphqlScalarValue::String(value.clone())),
        gql::Value::Int(value) => Ok(StudioGraphqlScalarValue::Int(value.as_i64().ok_or_else(
            || {
                format!(
                    "GraphQL field `{}` received out-of-range integer for `{argument_name}`",
                    field.name
                )
            },
        )?)),
        gql::Value::Float(value) => Ok(StudioGraphqlScalarValue::Float(*value)),
        other => Err(format!(
            "GraphQL field `{}` received unsupported scalar `{other}` for `{argument_name}`",
            field.name
        )),
    }
}
