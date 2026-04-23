use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use jsonschema::JSONSchema;
use serde_json::Value;

use crate::error::QianjiError;

use super::WendaoDocsContract;
use super::load::WendaoDocsParam;

impl WendaoDocsContract {
    pub(crate) fn validate_http_call(
        &self,
        method: &str,
        path: &str,
        query: &BTreeMap<String, Value>,
    ) -> Result<(), QianjiError> {
        self.ensure_task_type("http_call")?;
        if !self.snapshot.http.method.eq_ignore_ascii_case(method) {
            return Err(QianjiError::Topology(format!(
                "contract `{}` requires HTTP method `{}`, got `{method}`",
                self.snapshot.id, self.snapshot.http.method
            )));
        }
        validate_http_path(
            path,
            self.snapshot.http.path.as_str(),
            self.snapshot.id.as_str(),
        )?;

        let allowed = self
            .snapshot
            .http
            .query
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        for key in query.keys() {
            if !allowed.contains(key) {
                return Err(QianjiError::Topology(format!(
                    "contract `{}` does not allow HTTP query parameter `{key}`",
                    self.snapshot.id
                )));
            }
        }

        self.ensure_required_params(query.keys().map(String::as_str))?;
        self.validate_schema_instance(query)
    }

    pub(crate) fn validate_cli_call(&self, argv: &[String]) -> Result<(), QianjiError> {
        self.ensure_task_type("cli_call")?;
        let values = parse_cli_values(self, argv)?;
        self.ensure_required_params(values.keys().map(String::as_str))?;
        self.validate_schema_instance(&values)
    }

    fn ensure_task_type(&self, task_type: &str) -> Result<(), QianjiError> {
        if self
            .snapshot
            .task_types
            .iter()
            .any(|value| value == task_type)
        {
            return Ok(());
        }
        Err(QianjiError::Topology(format!(
            "contract `{}` does not support task type `{task_type}`",
            self.snapshot.id
        )))
    }

    fn ensure_required_params<'a>(
        &self,
        provided: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), QianjiError> {
        let provided = provided.into_iter().collect::<BTreeSet<_>>();
        for param in self.snapshot.params.iter().filter(|param| param.required) {
            if !provided.contains(param.name.as_str()) {
                return Err(QianjiError::Topology(format!(
                    "contract `{}` requires parameter `{}`",
                    self.snapshot.id, param.name
                )));
            }
        }
        Ok(())
    }

    fn validate_schema_instance(
        &self,
        values: &BTreeMap<String, Value>,
    ) -> Result<(), QianjiError> {
        let mut instance = serde_json::Map::new();
        let runtime_injected = self
            .snapshot
            .tool
            .runtime_injected
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for (name, value) in values {
            let param = self.param(name)?;
            if runtime_injected.contains(name.as_str()) {
                continue;
            }
            instance.insert(name.clone(), schema_probe_value(value, param)?);
        }

        let compiled = JSONSchema::options()
            .compile(&self.schema_json)
            .map_err(|error| {
                QianjiError::Topology(format!(
                    "failed to compile Wendao schema for `{}`: {error}",
                    self.snapshot.id
                ))
            })?;
        if let Err(errors) = compiled.validate(&Value::Object(instance)) {
            let joined = errors
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(QianjiError::Topology(format!(
                "contract `{}` rejected invocation values: {joined}",
                self.snapshot.id
            )));
        }
        Ok(())
    }

    fn param(&self, name: &str) -> Result<&WendaoDocsParam, QianjiError> {
        self.snapshot
            .params
            .iter()
            .find(|param| param.name == name)
            .ok_or_else(|| {
                QianjiError::Topology(format!(
                    "contract `{}` does not define parameter `{name}`",
                    self.snapshot.id
                ))
            })
    }
}

fn parse_cli_values(
    contract: &WendaoDocsContract,
    argv: &[String],
) -> Result<BTreeMap<String, Value>, QianjiError> {
    ensure_cli_prefix(
        argv,
        &contract.snapshot.cli.argv,
        contract.snapshot.id.as_str(),
    )?;
    let inverse_flags = contract
        .snapshot
        .cli
        .flags
        .iter()
        .map(|(param, flag)| (flag.as_str(), param.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut values = BTreeMap::new();
    let mut index = contract.snapshot.cli.argv.len();
    while index < argv.len() {
        let flag = argv[index].as_str();
        let Some(param_name) = inverse_flags.get(flag).copied() else {
            return Err(QianjiError::Topology(format!(
                "contract `{}` does not allow CLI flag `{flag}`",
                contract.snapshot.id
            )));
        };
        index += 1;
        let value = argv.get(index).ok_or_else(|| {
            QianjiError::Topology(format!(
                "contract `{}` CLI flag `{flag}` is missing a value",
                contract.snapshot.id
            ))
        })?;
        if values
            .insert(param_name.to_string(), Value::String(value.clone()))
            .is_some()
        {
            return Err(QianjiError::Topology(format!(
                "contract `{}` CLI flag `{flag}` must not be repeated",
                contract.snapshot.id
            )));
        }
        index += 1;
    }
    Ok(values)
}

fn ensure_cli_prefix(
    argv: &[String],
    expected_prefix: &[String],
    contract_id: &str,
) -> Result<(), QianjiError> {
    if argv.len() < expected_prefix.len() {
        return Err(QianjiError::Topology(format!(
            "contract `{contract_id}` requires CLI prefix `{}`",
            expected_prefix.join(" ")
        )));
    }
    for (index, expected) in expected_prefix.iter().enumerate() {
        let actual = argv[index].as_str();
        let matches = if index == 0 {
            actual == expected
                || Path::new(actual)
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .is_some_and(|file_name| file_name == expected)
        } else {
            actual == expected
        };
        if !matches {
            return Err(QianjiError::Topology(format!(
                "contract `{contract_id}` requires CLI prefix `{}`, got `{}`",
                expected_prefix.join(" "),
                argv.join(" ")
            )));
        }
    }
    Ok(())
}

fn validate_http_path(
    path: &str,
    expected_path: &str,
    contract_id: &str,
) -> Result<(), QianjiError> {
    match reqwest::Url::parse(path) {
        Ok(url) => {
            if url.path() != expected_path {
                return Err(QianjiError::Topology(format!(
                    "contract `{contract_id}` requires HTTP path `{expected_path}`, got `{}`",
                    url.path()
                )));
            }
        }
        Err(_) if path == expected_path => {}
        Err(_) => {
            return Err(QianjiError::Topology(format!(
                "contract `{contract_id}` requires HTTP path `{expected_path}`, got `{path}`"
            )));
        }
    }
    Ok(())
}

fn schema_probe_value(value: &Value, param: &WendaoDocsParam) -> Result<Value, QianjiError> {
    match value {
        Value::String(raw) if is_dynamic_placeholder(raw) => Ok(default_probe_value(param)),
        Value::String(raw) => coerce_literal_string(raw, param),
        Value::Number(_) if param.value_type == "integer" => Ok(value.clone()),
        Value::Bool(_) if param.value_type == "boolean" => Ok(value.clone()),
        Value::Null => Ok(Value::Null),
        _ => Err(QianjiError::Topology(format!(
            "parameter `{}` expected `{}` literal shape",
            param.name, param.value_type
        ))),
    }
}

fn coerce_literal_string(raw: &str, param: &WendaoDocsParam) -> Result<Value, QianjiError> {
    match param.value_type.as_str() {
        "string" => Ok(Value::String(raw.to_string())),
        "integer" => raw.parse::<u64>().map(Value::from).map_err(|_| {
            QianjiError::Topology(format!(
                "parameter `{}` expects an integer literal, got `{raw}`",
                param.name
            ))
        }),
        "boolean" => raw.parse::<bool>().map(Value::from).map_err(|_| {
            QianjiError::Topology(format!(
                "parameter `{}` expects a boolean literal, got `{raw}`",
                param.name
            ))
        }),
        other => Err(QianjiError::Topology(format!(
            "contract `{}` uses unsupported parameter type `{other}`",
            param.name
        ))),
    }
}

fn default_probe_value(param: &WendaoDocsParam) -> Value {
    match param.value_type.as_str() {
        "integer" => Value::from(0_u64),
        "boolean" => Value::from(false),
        _ => Value::String(String::new()),
    }
}

fn is_dynamic_placeholder(raw: &str) -> bool {
    let trimmed = raw.trim();
    trimmed.starts_with('$') || trimmed.contains("{{")
}
