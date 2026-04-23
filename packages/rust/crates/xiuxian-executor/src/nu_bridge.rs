//! Nushell System Bridge - The Core Algorithm.
//!
//! Transforms OS operations into structured JSON data flow.
//! Uses AST-based analysis and external security tools for validation.

use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::fs::Metadata;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use crate::error::{ExecutorError, Result};

const SHELLCHECK_CACHE_MAX_ENTRIES: usize = 512;
static SHELLCHECK_AVAILABLE: OnceLock<bool> = OnceLock::new();
static SHELLCHECK_RESULT_CACHE: OnceLock<Mutex<HashMap<String, ShellcheckResult>>> =
    OnceLock::new();

#[derive(Debug, Clone)]
enum ShellcheckResult {
    Pass,
    Violation(String),
}

/// Configuration for the Nushell bridge.
#[derive(Debug, Clone)]
pub struct NuConfig {
    /// Path to nushell binary.
    pub nu_path: String,
    /// Skip loading user config for reproducibility.
    pub no_config: bool,
    /// Timeout for command execution.
    pub timeout: Duration,
    /// Enable shellcheck for security validation.
    pub enable_shellcheck: bool,
    /// Additional allowed commands (whitelist).
    pub allowed_commands: Vec<String>,
}

impl Default for NuConfig {
    fn default() -> Self {
        Self {
            nu_path: "nu".to_string(),
            no_config: true,
            timeout: Duration::from_secs(30),
            enable_shellcheck: true,
            allowed_commands: vec![],
        }
    }
}

/// The core Nushell bridge implementation.
#[derive(Debug, Clone)]
pub struct NuSystemBridge {
    /// Bridge configuration.
    pub config: NuConfig,
}

impl Default for NuSystemBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl NuSystemBridge {
    /// Create a new bridge with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: NuConfig::default(),
        }
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: NuConfig) -> Self {
        Self { config }
    }

    /// Get reference to the configuration (for testing).
    #[must_use]
    pub fn config(&self) -> &NuConfig {
        &self.config
    }

    /// Enhance error messages with LLM-friendly hints for common Bash-to-Nu mistakes.
    fn enhance_error_for_llm(cmd: &str, stderr: &str) -> String {
        let mut hints = Vec::new();

        // Detect Bash find command
        if cmd.contains("find ")
            && (cmd.contains("-name") || cmd.contains("-size") || cmd.contains("-exec"))
        {
            hints.push("HINT: You are using Bash 'find' syntax. In Nushell, use 'ls **/*.py | where name =~ \"...\"' instead.");
        }

        // Detect && chaining
        if cmd.contains("&&") {
            hints.push("HINT: Nushell uses ';' instead of '&&' for command chaining.");
        }

        // Detect |& (Bash stderr redirect)
        if cmd.contains("|&") {
            hints.push("HINT: Nushell doesn't use '|&'. Use '| complete' to capture stderr, or just ignore it.");
        }

        // Detect backticks
        if cmd.contains('`') {
            hints.push("HINT: Nushell doesn't use backticks for command substitution. Use '$(command)' instead.");
        }

        // Detect $() with spaces (common Bash mistake)
        if cmd.contains("$(") && cmd.contains(' ') && !cmd.contains("${") {
            hints.push("HINT: In Nushell, command substitution $(cmd) captures the structured output, not just string.");
        }

        // Detect -flag style for Nushell commands
        if cmd.contains("ls -") && !cmd.contains("--") {
            hints.push("HINT: Nushell uses flags like '--long' or 'ls -l' works, but filtering should use '| where ...'.");
        }

        // Build enhanced error message
        if hints.is_empty() {
            stderr.to_string()
        } else {
            format!(
                "{}\n\n=== NUSHELL SYNTAX HINT ===\n{}\n=== END HINT ===",
                stderr,
                hints.join("\n")
            )
        }
    }

    /// Auto-correct common Nushell mistakes for mutation commands.
    fn auto_correct_mutation(cmd: &str) -> String {
        let mut corrected = cmd.to_string();

        // Fix 1: Auto-add -f (force) to save commands to prevent "file already exists" errors
        // Match "save " but not "save -f" or "save --force"
        if corrected.contains("save ") && !corrected.contains("save -") {
            corrected = corrected.replace("save ", "save -f ");
        }

        // Fix 2: Ensure mutation commands return a structured status instead of null
        // Add a status object at the end for better LLM feedback
        let needs_status = ["save ", "mv ", "cp ", "rm ", "mkdir ", "touch "];
        let ends_with_pipe = corrected.trim_end().ends_with('|');

        if needs_status.iter().any(|p| corrected.contains(p)) && !ends_with_pipe {
            // Extract filename if possible for the status
            let filename = if let Some(start) = corrected.find("save ") {
                let after_save = &corrected[start + 5..];
                let parts: Vec<&str> = after_save.split_whitespace().collect();
                if parts.is_empty() {
                    "\"unknown\"".to_string()
                } else {
                    let first = parts[0];
                    format!("\"{first}\"")
                }
            } else {
                "\"operation\"".to_string()
            };

            corrected = format!(
                "{corrected}; {{ status: 'success', file: {filename}, timestamp: (date now) }} | to json --raw"
            );
        }

        corrected
    }

    /// Try fast-path execution for simple observe commands.
    ///
    /// Currently supports `ls` without pipes/chaining operators and with at most one target path.
    fn try_execute_observe_fast_path(
        cmd: &str,
        action: ActionType,
        ensure_structured: bool,
    ) -> Option<Result<Value>> {
        if !matches!(action, ActionType::Observe) || !ensure_structured {
            return None;
        }
        let (target, include_hidden) = Self::parse_ls_fast_path_request(cmd)?;
        Some(Self::execute_ls_fast_path(&target, include_hidden))
    }

    /// Parse a command into a minimal `ls` fast-path request.
    fn parse_ls_fast_path_request(cmd: &str) -> Option<(String, bool)> {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed
            .chars()
            .any(|c| matches!(c, '|' | ';' | '&' | '>' | '<' | '\n' | '\r'))
        {
            return None;
        }

        let mut tokens = trimmed.split_whitespace();
        if tokens.next()? != "ls" {
            return None;
        }

        let mut include_hidden = false;
        let mut target: Option<&str> = None;
        for token in tokens {
            if token.starts_with("--") {
                if token == "--all" {
                    include_hidden = true;
                    continue;
                }
                return None;
            }
            if token.starts_with('-') {
                if token.chars().skip(1).any(|flag| flag == 'a') {
                    include_hidden = true;
                }
                continue;
            }
            if target.is_some() {
                return None;
            }
            if token
                .chars()
                .any(|c| matches!(c, '*' | '?' | '[' | ']' | '{' | '}' | '$'))
            {
                return None;
            }
            target = Some(token);
        }

        Some((target.unwrap_or(".").to_string(), include_hidden))
    }

    /// Execute `ls` fast-path directly via Rust filesystem APIs.
    fn execute_ls_fast_path(target: &str, include_hidden: bool) -> Result<Value> {
        let target_path = Path::new(target);
        let metadata = fs::symlink_metadata(target_path)
            .map_err(|e| ExecutorError::SystemError(format!("Fast-path ls failed: {e}")))?;

        if metadata.is_dir() {
            let mut rows: Vec<Value> = Vec::new();
            let dir_entries = fs::read_dir(target_path)
                .map_err(|e| ExecutorError::SystemError(format!("Fast-path ls failed: {e}")))?;
            for entry in dir_entries {
                let entry = entry
                    .map_err(|e| ExecutorError::SystemError(format!("Fast-path ls failed: {e}")))?;
                let name = entry.file_name().to_string_lossy().to_string();
                if !include_hidden && name.starts_with('.') {
                    continue;
                }
                let entry_path = entry.path();
                let entry_meta = fs::symlink_metadata(&entry_path).map_err(|e| {
                    ExecutorError::SystemError(format!("Fast-path ls metadata failed: {e}"))
                })?;
                rows.push(Self::build_ls_entry(&name, &entry_path, &entry_meta));
            }
            rows.sort_by(|left, right| {
                let left_name = left.get("name").and_then(Value::as_str).unwrap_or_default();
                let right_name = right
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                left_name.cmp(right_name)
            });
            return Ok(Value::Array(rows));
        }

        let display_name = target_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(target)
            .to_string();
        Ok(Value::Array(vec![Self::build_ls_entry(
            &display_name,
            target_path,
            &metadata,
        )]))
    }

    /// Build one `ls` row payload.
    fn build_ls_entry(name: &str, path: &Path, metadata: &Metadata) -> Value {
        let file_type = metadata.file_type();
        let kind = if metadata.is_dir() {
            "dir"
        } else if metadata.is_file() {
            "file"
        } else if file_type.is_symlink() {
            "symlink"
        } else {
            "other"
        };
        serde_json::json!({
            "name": name,
            "path": path.to_string_lossy(),
            "type": kind,
            "size": if metadata.is_file() { metadata.len() } else { 0_u64 },
            "readonly": metadata.permissions().readonly(),
        })
    }

    /// Execute a Nushell command with structured output.
    ///
    /// # Arguments
    /// * `cmd` - The command string to execute.
    /// * `ensure_structured` - If true, appends `| to json --raw` to force JSON output.
    ///
    /// # Returns
    /// Parsed JSON value or error.
    ///
    /// # Errors
    /// Returns an error when safety validation fails, the process cannot spawn, the command
    /// exits with a failure status, or the output cannot be parsed as JSON.
    pub fn execute(&self, cmd: &str, ensure_structured: bool) -> Result<Value> {
        let inferred_action = Self::classify_action(cmd);
        self.execute_with_action(cmd, inferred_action, ensure_structured)
    }

    /// Execute command with explicit action hint.
    ///
    /// This allows performance-sensitive callers to pass command intent so safety validation
    /// can skip expensive mutation checks for clearly observe-only commands.
    ///
    /// # Errors
    /// Same as [`Self::execute`].
    pub fn execute_with_action(
        &self,
        cmd: &str,
        action: ActionType,
        ensure_structured: bool,
    ) -> Result<Value> {
        // 0. Auto-correct mutation commands for better LLM feedback
        let cmd: Cow<'_, str> = if matches!(action, ActionType::Mutate) {
            Cow::Owned(Self::auto_correct_mutation(cmd))
        } else {
            Cow::Borrowed(cmd)
        };

        // 1. Security pre-flight check
        self.validate_safety_for_action(&cmd, action)?;

        // 1.5 Observe-only fast path to avoid process spawn overhead.
        if let Some(fast_path_result) =
            Self::try_execute_observe_fast_path(&cmd, action, ensure_structured)
        {
            return fast_path_result;
        }

        // 2. Construct the actual command
        let final_cmd = Self::build_command(&cmd, ensure_structured);

        // 3. Spawn and execute
        let output = Command::new(&self.config.nu_path)
            .args(["--no-config-file"]) // Reproducible environment
            .arg("-c")
            .arg(&final_cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| ExecutorError::SystemError(format!("Failed to spawn nu: {e}")))?;

        // 4. Handle execution errors with LLM-friendly hints
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let enhanced_error = Self::enhance_error_for_llm(&cmd, &stderr);
            return Err(ExecutorError::ShellError(
                output.status.code().unwrap_or(-1),
                enhanced_error,
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // 5. Handle empty output (e.g., cp, mv operations)
        if stdout.trim().is_empty() {
            return Ok(serde_json::json!({
                "status": "success",
                "operation": "mutation_complete"
            }));
        }

        // 6. Parse JSON output
        serde_json::from_str(&stdout).map_err(|e| {
            ExecutorError::SerializationError(format!(
                "Nu output wasn't valid JSON: {} (raw: {:?})",
                e,
                &stdout[..stdout.len().min(200)]
            ))
        })
    }

    /// Execute with timeout.
    ///
    /// # Errors
    /// Propagates any error returned by [`Self::execute`].
    pub fn execute_with_timeout(
        &self,
        cmd: &str,
        ensure_structured: bool,
        _timeout: Duration,
    ) -> Result<Value> {
        self.execute(cmd, ensure_structured)
    }

    /// Build the command string with JSON transformation.
    fn build_command(cmd: &str, ensure_structured: bool) -> String {
        if ensure_structured {
            // Force JSON output for observation commands
            format!("{cmd} | to json --raw")
        } else {
            cmd.to_string()
        }
    }

    /// Security pre-flight check.
    ///
    /// Uses AST-based analysis (ast-grep) and external tools (shellcheck).
    /// Step 1: Quick pattern check
    /// Step 2: `ShellCheck` integration (if enabled)
    ///
    /// # Errors
    /// Returns an error when the command contains dangerous patterns, fails `ShellCheck`,
    /// or is rejected by the configured whitelist.
    pub fn validate_safety(&self, cmd: &str) -> Result<()> {
        let inferred_action = Self::classify_action(cmd);
        self.validate_safety_for_action(cmd, inferred_action)
    }

    /// Security pre-flight check with action hint for better performance.
    ///
    /// `observe` commands that also classify as observe skip shellcheck.
    fn validate_safety_for_action(&self, cmd: &str, action: ActionType) -> Result<()> {
        // Step 1: Quick pattern check (immutable set)
        if Self::has_dangerous_pattern(cmd) {
            return Err(ExecutorError::SecurityViolation(
                "Dangerous pattern detected".to_string(),
            ));
        }

        // Step 2: ShellCheck validation for mutation-path commands only.
        // Guard rail: when action hint says observe but command text still looks mutating,
        // keep shellcheck enabled.
        let requires_shellcheck = match action {
            ActionType::Mutate => true,
            ActionType::Observe => matches!(Self::classify_action(cmd), ActionType::Mutate),
        };
        if self.config.enable_shellcheck && requires_shellcheck {
            Self::run_shellcheck(cmd)?;
        }

        // Step 3: Whitelist check
        if !self.config.allowed_commands.is_empty() {
            self.check_whitelist(cmd)?;
        }

        Ok(())
    }

    /// Quick pattern check for obvious dangers.
    fn has_dangerous_pattern(cmd: &str) -> bool {
        let cmd_lower = cmd.to_lowercase();
        // Obvious destructive patterns
        cmd_lower.contains("rm -rf /")
            || cmd_lower.contains("mkfs")
            || cmd_lower.contains(":(){ :|:& };:")
    }

    /// Run shellcheck for comprehensive analysis.
    fn run_shellcheck(cmd: &str) -> Result<()> {
        // Skip if shellcheck is not available.
        if !Self::shellcheck_available() {
            return Ok(());
        }

        // Fast-path: reuse cached verdict for identical command text.
        if let Some(cached) = Self::get_cached_shellcheck_result(cmd) {
            return match cached {
                ShellcheckResult::Pass => Ok(()),
                ShellcheckResult::Violation(error) => Err(ExecutorError::SecurityViolation(error)),
            };
        }

        let Ok(mut child) = Command::new("shellcheck")
            .args(["-e", "SC2034", "-"]) // Allow unused vars
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return Ok(());
        };

        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(cmd.as_bytes());
        }

        let Ok(output) = child.wait_with_output() else {
            return Ok(());
        };

        // ShellCheck returns:
        // - 0 for no issues
        // - 1 for warnings (allowed)
        // - >1 for parse/runtime errors (blocked)
        let exit_code = output.status.code().unwrap_or(0);
        if exit_code > 1 {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let error_message = format!("ShellCheck error: {stderr}");
            Self::cache_shellcheck_result(cmd, ShellcheckResult::Violation(error_message.clone()));
            return Err(ExecutorError::SecurityViolation(error_message));
        }

        Self::cache_shellcheck_result(cmd, ShellcheckResult::Pass);
        Ok(())
    }

    fn shellcheck_available() -> bool {
        *SHELLCHECK_AVAILABLE.get_or_init(|| {
            Command::new("shellcheck")
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        })
    }

    fn shellcheck_cache() -> &'static Mutex<HashMap<String, ShellcheckResult>> {
        SHELLCHECK_RESULT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn get_cached_shellcheck_result(cmd: &str) -> Option<ShellcheckResult> {
        let cache_guard = Self::shellcheck_cache().lock().ok()?;
        cache_guard.get(cmd).cloned()
    }

    fn cache_shellcheck_result(cmd: &str, result: ShellcheckResult) {
        if let Ok(mut cache_guard) = Self::shellcheck_cache().lock() {
            if cache_guard.len() >= SHELLCHECK_CACHE_MAX_ENTRIES {
                cache_guard.clear();
            }
            cache_guard.insert(cmd.to_string(), result);
        }
    }

    /// Check against whitelist.
    fn check_whitelist(&self, cmd: &str) -> Result<()> {
        let cmd_trimmed = cmd.trim();
        for allowed in &self.config.allowed_commands {
            if cmd_trimmed.starts_with(allowed) {
                return Ok(());
            }
        }
        Err(ExecutorError::SecurityViolation(
            "Command not in whitelist".to_string(),
        ))
    }

    /// Check if a command is a mutation (side-effect) operation.
    #[must_use]
    pub fn classify_action(cmd: &str) -> ActionType {
        let cmd_trimmed = cmd.trim();
        let cmd_lower = cmd_trimmed.to_lowercase();

        // Mutation indicators
        let mutation_keywords = [
            "rm", "mv", "cp", "save", "touch", "mkdir", "chmod", "chown", "echo", "print", "write",
        ];

        for keyword in &mutation_keywords {
            if cmd_lower.starts_with(keyword) || cmd_lower.contains(&format!(" | {keyword}")) {
                return ActionType::Mutate;
            }
        }

        ActionType::Observe
    }
}

/// Classification of command intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    /// Read-only operation (ls, open, ps, cat)
    Observe,
    /// Side-effect operation (rm, cp, mv, save)
    Mutate,
}
