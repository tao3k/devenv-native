use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use tempfile::TempDir;
use tokio::time::{Duration, Instant, sleep};
use xiuxian_qianji_bpmn_engine::{BpmnEngineError, load_checkpoint};

const VALKEY_SPAWN_ATTEMPTS: usize = 8;

pub(super) struct TestValkey {
    url: String,
    child: Option<Child>,
    temp_dir: Option<TempDir>,
    log_path: Option<PathBuf>,
}

impl TestValkey {
    pub(super) async fn spawn_if_available() -> anyhow::Result<Option<Self>> {
        if let Ok(url) = std::env::var("VALKEY_URL")
            && !url.trim().is_empty()
        {
            return Ok(Some(Self {
                url,
                child: None,
                temp_dir: None,
                log_path: None,
            }));
        }

        if !has_valkey_server_binary() {
            return Ok(None);
        }

        let mut last_port_conflict = None;
        for _attempt in 0..VALKEY_SPAWN_ATTEMPTS {
            let port = reserve_local_port()?;
            let temp_dir = tempfile::tempdir()?;
            let dir = temp_dir.path();
            let log_path = dir.join("valkey.log");
            let child = Command::new("valkey-server")
                .arg("--bind")
                .arg("127.0.0.1")
                .arg("--port")
                .arg(port.to_string())
                .arg("--save")
                .arg("")
                .arg("--appendonly")
                .arg("no")
                .arg("--dir")
                .arg(dir)
                .arg("--logfile")
                .arg(&log_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            let mut server = Self {
                url: format!("redis://127.0.0.1:{port}/0"),
                child: Some(child),
                temp_dir: Some(temp_dir),
                log_path: Some(log_path),
            };
            match server.wait_ready().await {
                Ok(()) => return Ok(Some(server)),
                Err(error) if is_port_conflict(&error) => {
                    last_port_conflict = Some(error);
                }
                Err(error) => return Err(error),
            }
        }

        if let Some(error) = last_port_conflict {
            return Err(error);
        }

        anyhow::bail!("valkey test server could not start after {VALKEY_SPAWN_ATTEMPTS} attempts")
    }

    pub(super) fn url(&self) -> &str {
        self.url.as_str()
    }

    async fn wait_ready(&mut self) -> anyhow::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if Instant::now() >= deadline {
                anyhow::bail!(
                    "timed out waiting for valkey test server at {}{}",
                    self.url,
                    self.log_suffix()
                );
            }

            if let Some(child) = self.child.as_mut()
                && let Some(status) = child.try_wait()?
            {
                anyhow::bail!(
                    "valkey test server exited with status {status}{}",
                    self.log_suffix()
                );
            }

            match load_checkpoint("wf_checkpoint", self.url()).await {
                Ok(_) => return Ok(()),
                Err(BpmnEngineError::CheckpointStorage { .. }) => {
                    sleep(Duration::from_millis(40)).await;
                }
                Err(other) => anyhow::bail!("unexpected checkpoint error while waiting: {other:?}"),
            }
        }
    }

    fn log_suffix(&self) -> String {
        self.log_path
            .as_deref()
            .and_then(read_log_excerpt)
            .map(|log| format!("; valkey log: {log}"))
            .unwrap_or_default()
    }
}

impl Drop for TestValkey {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut()
            && let Ok(None) = child.try_wait()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
        self.temp_dir = None;
    }
}

fn has_valkey_server_binary() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("command -v valkey-server >/dev/null 2>&1")
        .status()
        .is_ok_and(|status| status.success())
}

fn reserve_local_port() -> anyhow::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

fn is_port_conflict(error: &anyhow::Error) -> bool {
    let rendered = error.to_string();
    rendered.contains("Address already in use") || rendered.contains("Failed listening on port")
}

fn read_log_excerpt(path: &Path) -> Option<String> {
    let rendered = std::fs::read_to_string(path).ok()?;
    let trimmed = rendered.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(
        trimmed
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | "),
    )
}

#[test]
fn valkey_helper_classifies_port_conflicts_from_log_text() {
    let error = anyhow::anyhow!(
        "valkey test server exited with status exit status: 1; valkey log: Failed listening on port 58472 (tcp), aborting."
    );
    assert!(is_port_conflict(&error));
}

#[test]
fn valkey_helper_keeps_non_port_failures_distinct() {
    let error = anyhow::anyhow!("timed out waiting for valkey test server readiness");
    assert!(!is_port_conflict(&error));
}
