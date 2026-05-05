use std::io;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tracing_subscriber::fmt::writer::MakeWriter;
use xiuxian_daochang::Channel;

use super::support::{
    MockChannel, build_agent, inbound, process_discord_message, start_job_manager,
};

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

#[tokio::test(flavor = "current_thread")]
async fn process_discord_message_logs_structured_session_status_json_summary() -> Result<()> {
    let logs = SharedLogBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(logs.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session json"),
        &job_manager,
        10,
    )
    .await;

    let output = logs.as_string();
    assert!(output.contains("discord command reply sent"));
    assert!(output.contains("discord command reply json summary"));
    assert!(output.contains("discord.command.session_status_json.replied"));
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
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn process_discord_message_logs_structured_session_admin_json_summary() -> Result<()> {
    let logs = SharedLogBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_writer(logs.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let agent = build_agent().await?;
    let job_manager = start_job_manager(&agent);
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session admin list json"),
        &job_manager,
        10,
    )
    .await;

    let output = logs.as_string();
    assert!(output.contains("discord command reply json summary"));
    assert!(output.contains("discord.command.session_admin_json.replied"));
    assert!(output.contains("session_admin"));
    assert!(output.contains("json_audit_error"));
    assert!(output.contains("json_override_admin_count"));
    Ok(())
}
