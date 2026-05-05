//! Contract reference resolution and schema validation.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use quick_xml::Reader;
use quick_xml::events::Event;
use tempfile::NamedTempFile;

/// Contract validation failures for zhenfa-managed XML payloads.
#[derive(Debug, thiserror::Error)]
pub enum ZhenfaContractError {
    /// The referenced contract file does not exist on disk.
    #[error("zhenfa contract not found: {path}")]
    ContractNotFound {
        /// Missing contract path.
        path: PathBuf,
    },
    /// The contract file type is not supported by the validator.
    #[error("unsupported zhenfa contract type at {path}; expected an .xsd file")]
    UnsupportedContractType {
        /// Unsupported contract path.
        path: PathBuf,
    },
    /// The contract file could not be read.
    #[error("failed to read zhenfa contract {path}: {source}")]
    ReadContract {
        /// Contract path that failed to load.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: io::Error,
    },
    /// The contract file is not well-formed XML.
    #[error("invalid zhenfa contract XML at {path}: {message}")]
    InvalidContractXml {
        /// Contract path containing malformed XML.
        path: PathBuf,
        /// XML parsing diagnostic.
        message: String,
    },
    /// The XML payload is not well-formed before schema validation.
    #[error("invalid XML payload for zhenfa contract validation: {message}")]
    InvalidPayloadXml {
        /// XML parsing diagnostic.
        message: String,
    },
    /// The validator requires `xmllint` but could not find it on PATH.
    #[error("xmllint is required for XSD validation but was not found on PATH")]
    ValidatorUnavailable,
    /// The validator command could not be invoked.
    #[error("failed to invoke xmllint for zhenfa contract {path}: {source}")]
    ValidatorInvocation {
        /// Contract path passed to the validator.
        path: PathBuf,
        /// Underlying process spawn failure.
        #[source]
        source: io::Error,
    },
    /// The payload could not be staged to a temporary file for validation.
    #[error("failed to stage XML payload for zhenfa contract validation: {source}")]
    StagePayload {
        /// Underlying temporary-file IO failure.
        #[source]
        source: io::Error,
    },
    /// The XML payload failed XSD validation.
    #[error("xml payload does not satisfy zhenfa contract {path}: {message}")]
    ContractValidationFailed {
        /// Contract path used for schema validation.
        path: PathBuf,
        /// Validator diagnostic output.
        message: String,
    },
}

/// Resolve a contract path relative to a scenario or manifest directory.
#[must_use]
pub fn resolve_contract_path(
    contract_ref: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
) -> PathBuf {
    let contract_ref = contract_ref.as_ref();
    if contract_ref.is_absolute() {
        contract_ref.to_path_buf()
    } else {
        base_dir.as_ref().join(contract_ref)
    }
}

/// Validate one XML payload against one XSD contract on disk.
///
/// # Errors
///
/// Returns [`ZhenfaContractError`] when the contract path is missing, the XML
/// documents are malformed, `xmllint` is unavailable, or the payload violates
/// the declared XSD schema.
pub fn validate_contract(
    xml_payload: &str,
    contract_path: impl AsRef<Path>,
) -> Result<(), ZhenfaContractError> {
    let contract_path = contract_path.as_ref();
    if !contract_path.exists() {
        return Err(ZhenfaContractError::ContractNotFound {
            path: contract_path.to_path_buf(),
        });
    }
    if contract_path.extension() != Some(OsStr::new("xsd")) {
        return Err(ZhenfaContractError::UnsupportedContractType {
            path: contract_path.to_path_buf(),
        });
    }

    let contract_xml =
        fs::read_to_string(contract_path).map_err(|source| ZhenfaContractError::ReadContract {
            path: contract_path.to_path_buf(),
            source,
        })?;
    ensure_well_formed_xml(&contract_xml).map_err(|message| {
        ZhenfaContractError::InvalidContractXml {
            path: contract_path.to_path_buf(),
            message,
        }
    })?;
    ensure_well_formed_xml(xml_payload)
        .map_err(|message| ZhenfaContractError::InvalidPayloadXml { message })?;

    let mut staged_payload =
        NamedTempFile::new().map_err(|source| ZhenfaContractError::StagePayload { source })?;
    staged_payload
        .write_all(xml_payload.as_bytes())
        .and_then(|()| staged_payload.flush())
        .map_err(|source| ZhenfaContractError::StagePayload { source })?;

    let output = Command::new("xmllint")
        .arg("--noout")
        .arg("--schema")
        .arg(contract_path)
        .arg(staged_payload.path())
        .output()
        .map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                ZhenfaContractError::ValidatorUnavailable
            } else {
                ZhenfaContractError::ValidatorInvocation {
                    path: contract_path.to_path_buf(),
                    source,
                }
            }
        })?;

    if output.status.success() {
        return Ok(());
    }

    Err(ZhenfaContractError::ContractValidationFailed {
        path: contract_path.to_path_buf(),
        message: xmllint_message(&output.stderr, &output.stdout),
    })
}

/// Resolve one contract reference relative to `base_dir` and validate one XML payload.
///
/// # Errors
///
/// Returns [`ZhenfaContractError`] when the resolved contract path is invalid
/// or when the XML payload fails the resolved XSD validation.
pub fn validate_contract_reference(
    xml_payload: &str,
    contract_ref: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
) -> Result<PathBuf, ZhenfaContractError> {
    let resolved = resolve_contract_path(contract_ref, base_dir);
    validate_contract(xml_payload, &resolved)?;
    Ok(resolved)
}

fn ensure_well_formed_xml(xml: &str) -> Result<(), String> {
    let mut reader = Reader::from_str(xml);
    let mut saw_root = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(_) | Event::Empty(_)) => {
                saw_root = true;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(error.to_string()),
        }
    }

    if saw_root {
        Ok(())
    } else {
        Err("document has no root element".to_string())
    }
}

fn xmllint_message(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stdout = String::from_utf8_lossy(stdout);
    let message = stderr.trim();
    if !message.is_empty() {
        return message.to_string();
    }
    let message = stdout.trim();
    if !message.is_empty() {
        return message.to_string();
    }
    "schema validation failed without diagnostic output".to_string()
}
