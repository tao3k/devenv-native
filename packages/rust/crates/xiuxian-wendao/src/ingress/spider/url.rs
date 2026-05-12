//! `ingress::spider::url` owns Wendao ingress spider url behavior.

use url::Url;

use super::errors::SpiderIngressError;

/// Normalize one absolute web URL into canonical Wendao URI:
/// `wendao://web/<absolute_url>`.
///
/// # Errors
///
/// Returns [`SpiderIngressError::InvalidWebUrl`] or
/// [`SpiderIngressError::UnsupportedWebScheme`] when URL cannot be normalized.
pub fn canonical_web_uri(url: &str) -> Result<String, SpiderIngressError> {
    let normalized = parse_absolute_web_url(url)?;
    Ok(format!("wendao://web/{normalized}"))
}

/// Resolve semantic namespace from absolute web URL host.
///
/// # Errors
///
/// Returns [`SpiderIngressError`] when `url` is not a valid absolute `http(s)` URL.
pub fn web_namespace_from_url(url: &str) -> Result<String, SpiderIngressError> {
    let parsed = parse_absolute_web_url(url)?;
    let parsed = Url::parse(parsed.as_str()).map_err(|_| SpiderIngressError::InvalidWebUrl {
        url: url.to_string(),
    })?;
    parsed
        .host_str()
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| SpiderIngressError::InvalidWebUrl {
            url: url.to_string(),
        })
}

fn parse_absolute_web_url(url: &str) -> Result<String, SpiderIngressError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(SpiderIngressError::InvalidWebUrl {
            url: url.to_string(),
        });
    }

    let mut parsed = Url::parse(trimmed).map_err(|_| SpiderIngressError::InvalidWebUrl {
        url: trimmed.to_string(),
    })?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(SpiderIngressError::UnsupportedWebScheme {
            url: trimmed.to_string(),
            scheme: parsed.scheme().to_string(),
        });
    }
    if parsed.host_str().is_none() {
        return Err(SpiderIngressError::InvalidWebUrl {
            url: trimmed.to_string(),
        });
    }

    parsed.set_fragment(None);
    Ok(parsed.to_string())
}
