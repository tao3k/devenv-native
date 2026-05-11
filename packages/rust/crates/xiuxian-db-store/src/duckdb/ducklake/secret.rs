//! `DuckDB` `httpfs` secret SQL helpers for `DuckLake` remote data paths.

use serde::{Deserialize, Serialize};

use crate::duckdb::ensure_duckdb_identifier;

macro_rules! duckdb_secret_string_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Borrows the secret configuration value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

duckdb_secret_string_type!(
    /// Access key id for a DuckDB S3-compatible secret.
    DuckDbS3AccessKeyId
);
duckdb_secret_string_type!(
    /// Temporary session token for a DuckDB S3-compatible secret.
    DuckDbS3SessionToken
);

/// Credential provider for one `DuckDB` `S3`-compatible secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuckDbS3SecretProvider {
    /// Use explicit static credentials from the caller.
    Config {
        /// Access key id.
        key_id: DuckDbS3AccessKeyId,
        /// Secret access key.
        secret: String,
        /// Optional session token for temporary credentials.
        session_token: Option<DuckDbS3SessionToken>,
    },
    /// Use `DuckDB`'s credential-chain provider.
    CredentialChain {
        /// Optional chain name such as `config`.
        chain: Option<String>,
    },
}

/// SQL-renderable `DuckDB` `S3`-compatible secret configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuckDbS3SecretConfig {
    /// Secret name referenced by `DuckDB`.
    pub name: String,
    /// Credential provider.
    pub provider: DuckDbS3SecretProvider,
    /// Optional region.
    pub region: Option<String>,
    /// Optional custom endpoint for `S3`-compatible stores.
    pub endpoint: Option<String>,
    /// Optional URL style such as `path` or `vhost`.
    pub url_style: Option<String>,
    /// Optional scope such as `s3://bucket/prefix`.
    pub scope: Option<String>,
    /// Optional `SSL` toggle for local `S3`-compatible endpoints.
    pub use_ssl: Option<bool>,
    /// Whether rendered SQL should install/load the `httpfs` extension first.
    pub bootstrap_httpfs: bool,
}

impl DuckDbS3SecretConfig {
    /// Build a secret backed by explicit static credentials.
    #[must_use]
    pub fn config(
        name: impl Into<String>,
        key_id: impl Into<DuckDbS3AccessKeyId>,
        secret: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: DuckDbS3SecretProvider::Config {
                key_id: key_id.into(),
                secret: secret.into(),
                session_token: None,
            },
            region: None,
            endpoint: None,
            url_style: None,
            scope: None,
            use_ssl: None,
            bootstrap_httpfs: true,
        }
    }

    /// Build a secret backed by `DuckDB`'s credential chain.
    #[must_use]
    pub fn credential_chain(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            provider: DuckDbS3SecretProvider::CredentialChain { chain: None },
            region: None,
            endpoint: None,
            url_style: None,
            scope: None,
            use_ssl: None,
            bootstrap_httpfs: true,
        }
    }

    /// Set the optional region.
    #[must_use]
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set the optional endpoint.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set the optional URL style.
    #[must_use]
    pub fn with_url_style(mut self, url_style: impl Into<String>) -> Self {
        self.url_style = Some(url_style.into());
        self
    }

    /// Set the optional scope.
    #[must_use]
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Set the optional SSL flag.
    #[must_use]
    pub const fn with_use_ssl(mut self, use_ssl: bool) -> Self {
        self.use_ssl = Some(use_ssl);
        self
    }
}

/// Build SQL that installs/loads `httpfs` and creates one `S3`-compatible secret.
///
/// # Errors
///
/// Returns an error when the secret name is invalid or required credential
/// fields are blank.
pub fn build_duckdb_s3_secret_sql(config: &DuckDbS3SecretConfig) -> Result<String, String> {
    ensure_duckdb_identifier(&config.name, "DuckDB S3 secret")?;
    let mut statements = Vec::new();
    if config.bootstrap_httpfs {
        statements.push("INSTALL httpfs;\nLOAD httpfs;".to_string());
    }

    let mut entries = vec!["TYPE s3".to_string()];
    match &config.provider {
        DuckDbS3SecretProvider::Config {
            key_id,
            secret,
            session_token,
        } => {
            entries.push("PROVIDER config".to_string());
            push_non_blank_literal(&mut entries, "KEY_ID", key_id.as_str())?;
            push_non_blank_literal(&mut entries, "SECRET", secret)?;
            if let Some(session_token) = session_token {
                push_non_blank_literal(&mut entries, "SESSION_TOKEN", session_token.as_str())?;
            }
        }
        DuckDbS3SecretProvider::CredentialChain { chain } => {
            entries.push("PROVIDER credential_chain".to_string());
            if let Some(chain) = chain {
                ensure_duckdb_identifier(chain, "DuckDB S3 secret credential chain")?;
                entries.push(format!("CHAIN {chain}"));
            }
        }
    }
    if let Some(region) = &config.region {
        push_non_blank_literal(&mut entries, "REGION", region)?;
    }
    if let Some(endpoint) = &config.endpoint {
        push_non_blank_literal(&mut entries, "ENDPOINT", endpoint)?;
    }
    if let Some(url_style) = &config.url_style {
        push_non_blank_literal(&mut entries, "URL_STYLE", url_style)?;
    }
    if let Some(scope) = &config.scope {
        push_non_blank_literal(&mut entries, "SCOPE", scope)?;
    }
    if let Some(use_ssl) = config.use_ssl {
        entries.push(format!(
            "USE_SSL {}",
            if use_ssl { "true" } else { "false" }
        ));
    }

    statements.push(format!(
        "CREATE OR REPLACE SECRET {} (\n    {}\n);",
        config.name,
        entries.join(",\n    ")
    ));
    Ok(statements.join("\n"))
}

fn push_non_blank_literal(
    entries: &mut Vec<String>,
    keyword: &str,
    value: &str,
) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "DuckDB S3 secret field `{keyword}` cannot be blank"
        ));
    }
    entries.push(format!("{keyword} '{}'", trimmed.replace('\'', "''")));
    Ok(())
}
