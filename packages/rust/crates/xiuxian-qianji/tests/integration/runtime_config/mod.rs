//! Integration tests for qianji runtime config layering.

use std::fs;
use std::io;
use std::path::Path;
use tempfile::TempDir;
use xiuxian_qianji::runtime_config::{
    QianjiRuntimeCheckpointConfig, QianjiRuntimeEnv, QianjiRuntimeLlmConfig,
    QianjiRuntimeWendaoIngesterConfig, resolve_qianji_runtime_checkpoint_config_with_env,
    resolve_qianji_runtime_llm_config_with_env,
    resolve_qianji_runtime_wendao_ingester_config_with_env,
};

mod checkpoint;
mod llm_config;
mod wendao_ingester;

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!(
                "failed to create parent directory '{}': {err}",
                parent.display()
            )
        });
    }
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write file '{}': {err}", path.display()));
}

fn resolve(env: &QianjiRuntimeEnv) -> QianjiRuntimeLlmConfig {
    match resolve_qianji_runtime_llm_config_with_env(env) {
        Ok(cfg) => cfg,
        Err(err) => panic!("runtime config resolve should succeed: {err}"),
    }
}

fn resolve_wendao(env: &QianjiRuntimeEnv) -> QianjiRuntimeWendaoIngesterConfig {
    match resolve_qianji_runtime_wendao_ingester_config_with_env(env) {
        Ok(cfg) => cfg,
        Err(err) => panic!("runtime wendao config resolve should succeed: {err}"),
    }
}

fn resolve_checkpoint(env: &QianjiRuntimeEnv) -> QianjiRuntimeCheckpointConfig {
    match resolve_qianji_runtime_checkpoint_config_with_env(env) {
        Ok(cfg) => cfg,
        Err(err) => panic!("runtime checkpoint config resolve should succeed: {err}"),
    }
}

xiuxian_testing::crate_test_policy_harness!();
