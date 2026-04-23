use super::SurfaceColumn;

pub(super) fn render_sql_value(column: &SurfaceColumn, raw: &str) -> Result<String, String> {
    let data_type = normalize_token(column.data_type.as_str());
    if data_type.contains("bool") {
        return raw
            .parse::<bool>()
            .map(|value| value.to_string().to_ascii_uppercase())
            .map_err(|_| {
                format!(
                    "column `{}` expects a boolean value, received `{raw}`",
                    column.name
                )
            });
    }
    if data_type.contains("uint") {
        return raw
            .parse::<u64>()
            .map(|value| value.to_string())
            .map_err(|_| {
                format!(
                    "column `{}` expects an unsigned integer value, received `{raw}`",
                    column.name
                )
            });
    }
    if data_type.contains("int") {
        return raw
            .parse::<i64>()
            .map(|value| value.to_string())
            .map_err(|_| {
                format!(
                    "column `{}` expects an integer value, received `{raw}`",
                    column.name
                )
            });
    }
    if data_type.contains("float") || data_type.contains("double") || data_type.contains("decimal")
    {
        return raw
            .parse::<f64>()
            .map_err(|_| {
                format!(
                    "column `{}` expects a numeric value, received `{raw}`",
                    column.name
                )
            })
            .and_then(|value| {
                if value.is_finite() {
                    Ok(value.to_string())
                } else {
                    Err(format!(
                        "column `{}` expects a finite numeric value",
                        column.name
                    ))
                }
            });
    }
    Ok(quote_string_literal(raw))
}

pub(super) fn quote_string_literal(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', "''"))
}

pub(super) fn is_text_like(column: &SurfaceColumn) -> bool {
    let data_type = normalize_token(column.data_type.as_str());
    data_type.contains("utf8") || data_type.contains("string") || data_type.contains("text")
}

pub(super) fn normalize_token(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}
