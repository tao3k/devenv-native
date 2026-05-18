use std::path::Path;

use super::{
    absolute_runtime_path, docling_document_analyzer_command_spec, image_ocr_analyzer_command_spec,
};

mod commands;

fn expected_args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
