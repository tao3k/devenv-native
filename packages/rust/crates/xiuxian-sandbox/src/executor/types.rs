//! Core sandbox types and shared execution helpers.

use serde::Deserialize;
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::process::Command as AsyncCommand;
use tokio::time::{Duration, timeout};

/// Execution result returned by sandbox backends.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether the execution succeeded.
    pub success: bool,
    /// Exit code from the process (if available).
    pub exit_code: Option<i32>,
    /// Standard output captured from the process.
    pub stdout: String,
    /// Standard error captured from the process.
    pub stderr: String,
    /// Execution time in milliseconds.
    pub execution_time_ms: u64,
    /// Optional memory usage in bytes.
    pub memory_used_bytes: Option<u64>,
    /// Optional error message when execution failed unexpectedly.
    pub error: Option<String>,
}

/// Mount configuration for sandboxed execution.
#[derive(Debug, Clone)]
pub struct MountConfig {
    /// Source path on the host.
    pub src: String,
    /// Destination path inside the sandbox.
    pub dst: String,
    /// Filesystem type (e.g. "bind", "tmpfs").
    pub fstype: String,
    /// Whether the mount is read-write.
    pub rw: bool,
}

/// Sandbox configuration shared across backends.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Skill identifier for logging.
    pub skill_id: String,
    /// Execution mode string (e.g. "EXEC").
    pub mode: SandboxMode,
    /// Sandbox hostname.
    pub hostname: String,
    /// Command to execute.
    pub cmd: Vec<String>,
    /// Environment variables (KEY=VALUE).
    pub env: Vec<String>,
    /// Mount configuration list.
    pub mounts: Vec<MountConfig>,
    /// Address space limit in bytes.
    pub rlimit_as: u64,
    /// CPU time limit in seconds.
    pub rlimit_cpu: u64,
    /// File size limit in bytes.
    pub rlimit_fsize: u64,
    /// Seccomp mode.
    pub seccomp_mode: u32,
    /// Log level string for the backend.
    pub log_level: String,
}

/// Typed sandbox execution mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct SandboxMode(String);

impl SandboxMode {
    /// Create a sandbox mode from a string-like value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the mode as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SandboxMode {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SandboxMode {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq<&str> for SandboxMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Sandbox executor trait for backend implementations.
#[async_trait::async_trait]
pub trait SandboxExecutor: Send + Sync {
    /// Backend name.
    fn name(&self) -> &'static str;

    /// Execute the sandbox with a configuration file and optional input.
    ///
    /// # Errors
    ///
    /// Returns an error string if the sandbox process fails to launch or the
    /// output cannot be collected.
    async fn execute(&self, config_path: &Path, input: &str) -> Result<ExecutionResult, String>;
}

/// Execute a command with time and memory limits.
///
/// # Errors
///
/// Returns an error string if the process cannot be spawned or its output
/// cannot be collected.
pub(super) async fn execute_with_limits(
    mut cmd: AsyncCommand,
    timeout_secs: u64,
    _memory_limit_bytes: u64,
) -> Result<ExecutionResult, String> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let start = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {e}"))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture stderr".to_string())?;

    let stdout_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).await.map(|_| buf)
    });
    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stderr.read_to_end(&mut buf).await.map(|_| buf)
    });

    let status = if timeout_secs == 0 {
        child
            .wait()
            .await
            .map_err(|e| format!("Failed to collect status: {e}"))?
    } else if let Ok(result) = timeout(Duration::from_secs(timeout_secs), child.wait()).await {
        result.map_err(|e| format!("Failed to collect status: {e}"))?
    } else {
        let _ = child.kill().await;
        stdout_task.abort();
        stderr_task.abort();
        return Ok(ExecutionResult {
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            execution_time_ms: duration_to_millis(start.elapsed()),
            memory_used_bytes: None,
            error: Some("Execution timed out".to_string()),
        });
    };

    let stdout_bytes = stdout_task
        .await
        .map_err(|e| format!("stdout task failed: {e}"))?
        .map_err(|e| format!("Failed to read stdout: {e}"))?;
    let stderr_bytes = stderr_task
        .await
        .map_err(|e| format!("stderr task failed: {e}"))?
        .map_err(|e| format!("Failed to read stderr: {e}"))?;

    let elapsed_ms = duration_to_millis(start.elapsed());
    Ok(ExecutionResult {
        success: status.success(),
        exit_code: status.code(),
        stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
        stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
        execution_time_ms: elapsed_ms,
        memory_used_bytes: None,
        error: None,
    })
}

fn duration_to_millis(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
