//! JSON filter conversion for Lance SQL-like WHERE clauses.

/// Convert JSON filter expression to `LanceDB` WHERE clause.
#[must_use]
pub fn json_to_lance_where(expr: &serde_json::Value) -> String {
    match expr {
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                return String::new();
            }
            let mut clauses = Vec::new();
            for (key, value) in obj {
                let clause = match value {
                    serde_json::Value::Object(comp) => {
                        if let Some(op) = comp.keys().next() {
                            match op.as_str() {
                                "$gt" | ">" => {
                                    if let Some(val) = comp.get("$gt").or(comp.get(">")) {
                                        match val {
                                            serde_json::Value::String(s) => {
                                                format!("{key} > '{s}'")
                                            }
                                            _ => format!("{key} > {val}"),
                                        }
                                    } else {
                                        continue;
                                    }
                                }
                                "$gte" | ">=" => {
                                    if let Some(val) = comp.get("$gte").or(comp.get(">=")) {
                                        match val {
                                            serde_json::Value::String(s) => {
                                                format!("{key} >= '{s}'")
                                            }
                                            _ => format!("{key} >= {val}"),
                                        }
                                    } else {
                                        continue;
                                    }
                                }
                                "$lt" | "<" => {
                                    if let Some(val) = comp.get("$lt").or(comp.get("<")) {
                                        match val {
                                            serde_json::Value::String(s) => {
                                                format!("{key} < '{s}'")
                                            }
                                            _ => format!("{key} < {val}"),
                                        }
                                    } else {
                                        continue;
                                    }
                                }
                                "$lte" | "<=" => {
                                    if let Some(val) = comp.get("$lte").or(comp.get("<=")) {
                                        match val {
                                            serde_json::Value::String(s) => {
                                                format!("{key} <= '{s}'")
                                            }
                                            _ => format!("{key} <= {val}"),
                                        }
                                    } else {
                                        continue;
                                    }
                                }
                                "$ne" | "!=" => {
                                    if let Some(val) = comp.get("$ne").or(comp.get("!=")) {
                                        match val {
                                            serde_json::Value::String(s) => {
                                                format!("{key} != '{s}'")
                                            }
                                            _ => format!("{key} != {val}"),
                                        }
                                    } else {
                                        continue;
                                    }
                                }
                                _ => continue,
                            }
                        } else {
                            continue;
                        }
                    }
                    serde_json::Value::String(s) => format!("{key} = '{s}'"),
                    serde_json::Value::Number(n) => format!("{key} = {n}"),
                    serde_json::Value::Bool(b) => format!("{key} = {b}"),
                    _ => continue,
                };
                clauses.push(clause);
            }
            if clauses.is_empty() {
                String::new()
            } else {
                clauses.join(" AND ")
            }
        }
        _ => String::new(),
    }
}
