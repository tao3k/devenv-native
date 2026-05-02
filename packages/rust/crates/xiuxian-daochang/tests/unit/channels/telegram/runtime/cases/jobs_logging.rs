//! Telegram runtime jobs logging tests for structured preview traces.

use std::io;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::sync::mpsc;
use tracing_subscriber::fmt::writer::MakeWriter;

use super::{MockChannel, build_agent, build_job_manager, handle_inbound_message, inbound};
use xiuxian_daochang::test_support::telegram_log_preview as log_preview;
use xiuxian_daochang::{Channel, ChannelMessage};

#[derive(Clone, Default)]
struct SharedLogBuffer {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl SharedLogBuffer {
    fn as_string(&self) -> String {
        match self.inner.lock() {
            Ok(guard) => String::from_utf8_lossy(&guard).to_string(),
            Err(_) => String::new(),
        }
    }
}

struct SharedLogWriter {
    inner: Arc<Mutex<Vec<u8>>>,
}

impl<'a> MakeWriter<'a> for SharedLogBuffer {
    type Writer = SharedLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        SharedLogWriter {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl io::Write for SharedLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(mut guard) = self.inner.lock() {
            guard.extend_from_slice(buf);
            Ok(buf.len())
        } else {
            Err(io::Error::other("failed to lock shared log buffer"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn runtime_jobs_log_preview_flattens_newlines_and_truncates() {
    let input = format!("line1\n{} ACK SQLITE", "x".repeat(120));
    let preview = log_preview(&input);
    assert!(!preview.contains('\n'));
    assert!(preview.starts_with("line1 "));
    assert!(preview.contains("..."));
    assert!(preview.contains("ACK SQLITE"));
}

#[test]
fn runtime_jobs_log_preview_keeps_short_messages() {
    let preview = log_preview("short status");
    assert_eq!(preview, "short status");
}

#[test]
fn runtime_jobs_log_preview_strips_think_block_and_keeps_visible_answer() {
    let preview = log_preview("<think>internal reasoning should not leak</think>\nACK SQLITE");
    assert!(!preview.contains("internal reasoning"));
    assert_eq!(preview.trim(), "ACK SQLITE");
}

#[test]
fn runtime_jobs_log_preview_preserves_memory_anchor_when_middle_truncated() {
    let input = [
        "## Session Memory",
        "Captured at unix ms: `1771887827965`",
        "- Session scope: `telegram:1304799691`",
        "### Trigger - Decision",
        "- `decision=skipped` `query_tokens=32` `pipeline_ms=5`",
        "### Recall Result",
        "- `injected=0` / `selected=0` / `total=0`",
        "Tip: run `/session memory json` for full payload.",
    ]
    .join("\n");
    let preview = log_preview(&input);
    assert!(preview.contains("Session Memory"));
    assert!(preview.contains("Trigger") || preview.contains("Recall Result"));
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_jobs_logs_structured_command_reply_event() -> Result<()> {
    let logs = SharedLogBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(logs.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    assert!(
        handle_inbound_message(
            inbound("/session json"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(foreground_rx.try_recv().is_err());

    let output = logs.as_string();
    assert!(output.contains("telegram command reply sent"));
    assert!(output.contains("telegram command reply json summary"));
    assert!(output.contains("telegram.command.session_status_json.replied"));
    assert!(output.contains("json_kind"));
    assert!(output.contains("session_context"));
    assert!(output.contains("json_audit_error"));
    assert!(output.contains("json_session_scope"));
    assert!(output.contains("json_logical_session_id"));
    assert!(output.contains("json_partition_key"));
    assert!(output.contains("json_admission_enabled"));
    assert!(output.contains("json_admission_total"));
    assert!(output.contains("json_admission_rejected"));
    assert!(output.contains("json_keys="));
    assert!(output.contains("session_key"));
    assert!(output.contains("-200:888"));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_jobs_logs_structured_memory_json_summary_fields() -> Result<()> {
    let logs = SharedLogBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(logs.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    let message = ChannelMessage {
        id: "jobs-logging-memory-json".to_string(),
        sender: "888".to_string(),
        recipient: "-200".to_string(),
        session_key: "-200:888:jobs-logging-memory-json".to_string(),
        content: "/session memory json".to_string(),
        attachments: Vec::new(),
        channel: "telegram".to_string(),
        timestamp: 0,
    };
    assert!(
        handle_inbound_message(message, &channel_dyn, &foreground_tx, &job_manager, &agent).await
    );
    assert!(foreground_rx.try_recv().is_err());

    let output = logs.as_string();
    assert!(output.contains("telegram command reply json summary"));
    assert!(output.contains("telegram.command.session_memory_json.replied"));
    assert!(output.contains("json_kind"));
    assert!(output.contains("session_memory"));
    assert!(output.contains("json_audit_error"));
    assert!(output.contains("json_available=false") || output.contains("json_available=\"false\""));
    assert!(
        output.contains("json_status=not_found") || output.contains("json_status=\"not_found\"")
    );
    assert!(output.contains("json_session_scope"));
    assert!(output.contains("telegram:-200:888:jobs-logging-memory-json"));
    assert!(output.contains("json_runtime_backend_ready"));
    assert!(output.contains("json_runtime_startup_load_status"));
    assert!(output.contains("json_admission_enabled"));
    assert!(output.contains("json_admission_total"));
    assert!(output.contains("json_admission_rejected"));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_jobs_logs_structured_session_admin_json_summary_fields() -> Result<()> {
    let logs = SharedLogBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(logs.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    assert!(
        handle_inbound_message(
            inbound("/session admin list json"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(foreground_rx.try_recv().is_err());

    let output = logs.as_string();
    assert!(output.contains("telegram command reply json summary"));
    assert!(output.contains("telegram.command.session_admin_json.replied"));
    assert!(output.contains("session_admin"));
    assert!(output.contains("json_audit_error"));
    assert!(output.contains("json_override_admin_count"));
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_jobs_logs_structured_reset_snapshot_state_event() -> Result<()> {
    let logs = SharedLogBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(logs.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    assert!(
        handle_inbound_message(
            inbound("/reset"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(foreground_rx.try_recv().is_err());

    let output = logs.as_string();
    assert!(output.contains("telegram command reset snapshot state"));
    assert!(output.contains("telegram.command.session_reset.snapshot_state"));
    assert!(output.contains("snapshot_state=none") || output.contains("snapshot_state=\"none\""));
    assert!(output.contains("session_key"));
    assert!(output.contains("-200:888"));
    Ok(())
}
