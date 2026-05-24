use std::path::Path;

use super::{
    external::{
        docling_document_analyzer_command_spec, image_ocr_analyzer_command_spec,
        should_skip_analyzer,
    },
    handler::{
        DEFAULT_EPISTEME_OPENAI_COMPATIBLE_PROMPT_AUDIT_MODEL, openai_compatible_prompt_audit_model,
    },
    root::{absolute_runtime_path, resolve_legacy_office_converter},
};
use xiuxian_wendao::episteme::EpistemeRuntimeConfig;

mod commands;
mod prompt_audit_model;

fn expected_args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
