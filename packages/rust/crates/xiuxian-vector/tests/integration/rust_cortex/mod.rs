//! Tests for Rust-native cortex tool search surfaces.

use anyhow::Result;
use xiuxian_vector::{
    AgenticSearchConfig, QueryIntent, ToolSearchOptions, ToolSearchRequest, VectorStore,
};

type KeywordDoc = (String, String, String, Vec<String>, Vec<String>);

fn clean_test_db(path: &std::path::Path) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}

mod agentic_search;
mod catalog;
mod command_resolution;
mod intent;
mod keyword_backend;
