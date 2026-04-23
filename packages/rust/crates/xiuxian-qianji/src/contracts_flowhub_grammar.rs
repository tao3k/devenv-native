use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Parsed `template.use` item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateUseSpec {
    /// Selected module reference, which may be hierarchical.
    pub module_ref: String,
    /// Alias assigned inside the scenario.
    pub alias: String,
}

impl fmt::Display for TemplateUseSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} as {}", self.module_ref, self.alias)
    }
}

impl FromStr for TemplateUseSpec {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = raw.split_whitespace().collect();
        if tokens.len() != 3 || tokens[1] != "as" {
            return Err(format!(
                "invalid template.use item `{raw}`: expected `<module-ref> as <alias>`"
            ));
        }

        let module_ref = tokens[0].trim();
        let alias = tokens[2].trim();
        if module_ref.is_empty() || alias.is_empty() {
            return Err(format!(
                "invalid template.use item `{raw}`: expected non-empty module reference and alias"
            ));
        }
        validate_template_module_ref(module_ref, raw)?;
        validate_template_alias(alias, raw)?;

        Ok(Self {
            module_ref: module_ref.to_string(),
            alias: alias.to_string(),
        })
    }
}

impl Serialize for TemplateUseSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for TemplateUseSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(D::Error::custom)
    }
}

/// Parsed `<alias>::<symbol>` reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateLinkRef {
    /// Declared alias from `[template].use`, when the reference targets a
    /// child or scenario-selected module.
    pub alias: Option<String>,
    /// Export or stable node symbol under the alias.
    pub symbol: String,
}

impl fmt::Display for TemplateLinkRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(alias) = &self.alias {
            write!(f, "{alias}::{}", self.symbol)
        } else {
            write!(f, "{}", self.symbol)
        }
    }
}

impl FromStr for TemplateLinkRef {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let separator_count = raw.matches("::").count();
        if separator_count > 1 {
            return Err(format!(
                "invalid template.link reference `{raw}`: expected `<alias>::<symbol>` or `<symbol>`"
            ));
        }

        if separator_count == 1 {
            let (alias, symbol) = raw.split_once("::").ok_or_else(|| {
                format!(
                    "invalid template.link reference `{raw}`: expected `<alias>::<symbol>` or `<symbol>`"
                )
            })?;
            let alias = alias.trim();
            let symbol = symbol.trim();
            if alias.is_empty() || symbol.is_empty() {
                return Err(format!(
                    "invalid template.link reference `{raw}`: expected non-empty alias and symbol"
                ));
            }
            validate_template_link_alias(alias, raw)?;
            validate_template_link_symbol(symbol, raw)?;

            return Ok(Self {
                alias: Some(alias.to_string()),
                symbol: symbol.to_string(),
            });
        }

        let symbol = raw.trim();
        if symbol.is_empty() {
            return Err(format!(
                "invalid template.link reference `{raw}`: expected non-empty symbol"
            ));
        }
        validate_template_link_symbol(symbol, raw)?;

        Ok(Self {
            alias: None,
            symbol: symbol.to_string(),
        })
    }
}

impl Serialize for TemplateLinkRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for TemplateLinkRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(D::Error::custom)
    }
}

/// One cross-module link declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateLinkSpec {
    /// Source alias-qualified symbol.
    pub from: TemplateLinkRef,
    /// Destination alias-qualified symbol.
    pub to: TemplateLinkRef,
}

fn validate_template_module_ref(module_ref: &str, raw: &str) -> Result<(), String> {
    if module_ref.starts_with('/') || module_ref.ends_with('/') || module_ref.contains('\\') {
        return Err(format!(
            "invalid template.use item `{raw}`: module reference `{module_ref}` must stay within the Flowhub hierarchy"
        ));
    }

    for segment in module_ref.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(format!(
                "invalid template.use item `{raw}`: module reference `{module_ref}` contains an invalid path segment"
            ));
        }
        if segment.contains("::") || segment.chars().any(char::is_whitespace) {
            return Err(format!(
                "invalid template.use item `{raw}`: module reference `{module_ref}` contains an invalid path segment"
            ));
        }
    }

    Ok(())
}

fn validate_template_alias(alias: &str, raw: &str) -> Result<(), String> {
    if is_plain_alias_identifier(alias) {
        return Ok(());
    }

    Err(format!(
        "invalid template.use item `{raw}`: alias `{alias}` must be a plain first-level identifier"
    ))
}

fn validate_template_link_alias(alias: &str, raw: &str) -> Result<(), String> {
    if !is_plain_alias_identifier(alias) {
        return Err(format!(
            "invalid template.link reference `{raw}`: alias `{alias}` must be a plain first-level identifier"
        ));
    }

    Ok(())
}

fn validate_template_link_symbol(symbol: &str, raw: &str) -> Result<(), String> {
    if symbol.chars().any(char::is_whitespace) {
        return Err(format!(
            "invalid template.link reference `{raw}`: symbol `{symbol}` must not contain whitespace"
        ));
    }

    Ok(())
}

fn is_plain_alias_identifier(value: &str) -> bool {
    !matches!(value, "." | "..")
        && !value.is_empty()
        && !value.contains('/')
        && !value.contains('\\')
        && !value.contains("::")
        && !value.chars().any(char::is_whitespace)
}
