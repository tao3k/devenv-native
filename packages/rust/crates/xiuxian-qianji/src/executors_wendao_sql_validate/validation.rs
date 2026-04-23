use super::render_sql::{is_text_like, normalize_token, quote_string_literal, render_sql_value};
use super::{SqlAuthorSpec, SqlFilter, SqlOrderTerm, SurfaceBundle};

pub(super) fn validate_and_render_sql(
    bundle: &SurfaceBundle,
    spec: &SqlAuthorSpec,
) -> Result<String, String> {
    let object = bundle
        .find_object(spec.target_object.as_str())
        .ok_or_else(|| {
            format!(
                "target object `{}` is not exposed by the surface bundle",
                spec.target_object
            )
        })?;

    if spec.projection.is_empty() {
        return Err("projection must include at least one column".to_string());
    }
    if spec.projection.iter().any(|column| column == "*") {
        return Err("SELECT * is not allowed".to_string());
    }
    if spec.limit == 0 {
        return Err("limit must be greater than zero".to_string());
    }
    if spec.limit > bundle.policy.max_limit {
        return Err(format!(
            "limit {} exceeds max_limit {}",
            spec.limit, bundle.policy.max_limit
        ));
    }
    if bundle.policy.requires_filter_for(object.name.as_str()) && spec.filters.is_empty() {
        return Err(format!(
            "object `{}` requires at least one narrowing filter",
            object.name
        ));
    }

    let projection = spec
        .projection
        .iter()
        .map(|column| {
            object.find_column(column.as_str()).ok_or_else(|| {
                format!(
                    "projection column `{column}` is not exposed for `{}`",
                    object.name
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let filters = spec
        .filters
        .iter()
        .map(|filter| validate_filter(filter, object, bundle))
        .collect::<Result<Vec<_>, _>>()?;

    let order_by = spec
        .order_by
        .iter()
        .map(|term| validate_order_term(term, object))
        .collect::<Result<Vec<_>, _>>()?;

    let mut sql = format!(
        "SELECT {} FROM {}",
        projection
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        object.name
    );
    if !filters.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(filters.join(" AND ").as_str());
    }
    if !order_by.is_empty() {
        sql.push_str(" ORDER BY ");
        sql.push_str(order_by.join(", ").as_str());
    }
    sql.push_str(format!(" LIMIT {}", spec.limit).as_str());
    Ok(sql)
}

fn validate_filter(
    filter: &SqlFilter,
    object: &super::super::contract::SurfaceObject,
    bundle: &SurfaceBundle,
) -> Result<String, String> {
    let column = object.find_column(filter.column.as_str()).ok_or_else(|| {
        format!(
            "filter column `{}` is not exposed for `{}`",
            filter.column, object.name
        )
    })?;
    if !bundle.policy.allows_op(filter.op.as_str()) {
        return Err(format!("filter op `{}` is not allowed", filter.op));
    }

    match normalize_token(filter.op.as_str()).as_str() {
        "eq" => Ok(format!(
            "{} = {}",
            column.name,
            render_sql_value(column, filter.value.as_str())?
        )),
        "contains" => {
            if !is_text_like(column) {
                return Err(format!(
                    "filter op `contains` is only allowed on text columns; `{}` has type `{}`",
                    column.name, column.data_type
                ));
            }
            Ok(format!(
                "{} LIKE {}",
                column.name,
                quote_string_literal(format!("%{}%", filter.value).as_str())
            ))
        }
        _ => Err(format!("unsupported filter op `{}`", filter.op)),
    }
}

fn validate_order_term(
    term: &SqlOrderTerm,
    object: &super::super::contract::SurfaceObject,
) -> Result<String, String> {
    let column = object.find_column(term.column.as_str()).ok_or_else(|| {
        format!(
            "order_by column `{}` is not exposed for `{}`",
            term.column, object.name
        )
    })?;
    let direction = normalize_token(term.direction.as_str());
    match direction.as_str() {
        "asc" => Ok(format!("{} ASC", column.name)),
        "desc" => Ok(format!("{} DESC", column.name)),
        _ => Err(format!(
            "order_by direction `{}` must be `asc` or `desc`",
            term.direction
        )),
    }
}
