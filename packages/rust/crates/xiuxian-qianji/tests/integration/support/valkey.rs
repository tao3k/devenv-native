use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result, bail};
use tempfile::TempDir;
use tokio::time::{Duration, Instant, sleep};

pub(crate) struct TestValkey {
    url: String,
    child: Option<Child>,
    temp_dir: Option<TempDir>,
    log_path: Option<PathBuf>,
}

impl TestValkey {
    pub(crate) async fn spawn() -> Result<Self> {
        if let Ok(url) = std::env::var("VALKEY_URL")
            && !url.trim().is_empty()
        {
            return Ok(Self {
                url,
                child: None,
                temp_dir: None,
                log_path: None,
            });
        }

        let port = reserve_local_port()?;
        let temp_dir = tempfile::tempdir().context("failed to create temp dir for valkey")?;
        let dir = temp_dir.path();
        let log_path = dir.join("valkey.log");
        let server_bin = resolve_valkey_server_binary()?;
        let child = Command::new(&server_bin)
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
            .spawn()
            .with_context(|| {
                format!(
                    "failed to spawn valkey-server for test via {}",
                    server_bin.display()
                )
            })?;
        let url = format!("redis://127.0.0.1:{port}/0");

        let mut server = Self {
            url,
            child: Some(child),
            temp_dir: Some(temp_dir),
            log_path: Some(log_path),
        };
        server.wait_ready().await?;
        Ok(server)
    }

    pub(crate) fn url(&self) -> &str {
        self.url.as_str()
    }

    async fn wait_ready(&mut self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if Instant::now() >= deadline {
                bail!(
                    "timed out waiting for valkey test server at {}{}",
                    self.url,
                    self.log_suffix()
                );
            }

            if let Some(child) = self.child.as_mut()
                && let Some(status) = child.try_wait().context("failed to poll valkey child")?
            {
                bail!(
                    "valkey test server exited with status {status}{}",
                    self.log_suffix()
                );
            }

            match redis::Client::open(self.url.clone())
                .context("failed to open redis client for valkey test server")?
                .get_multiplexed_async_connection()
                .await
            {
                Ok(_) => return Ok(()),
                Err(_) => sleep(Duration::from_millis(40)).await,
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

fn reserve_local_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").context("failed to reserve local port")?;
    let port = listener
        .local_addr()
        .context("failed to read reserved local port")?
        .port();
    Ok(port)
}

fn resolve_valkey_server_binary() -> Result<PathBuf> {
    if let Some(explicit) = std::env::var_os("VALKEY_SERVER_BIN")
        && !explicit.is_empty()
    {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Ok(path);
        }
        bail!(
            "VALKEY_SERVER_BIN points to {}, but that file does not exist",
            path.display()
        );
    }

    let Some(path_var) = std::env::var_os("PATH") else {
        bail!("PATH is not set and VALKEY_SERVER_BIN was not provided");
    };
    for entry in std::env::split_paths(&path_var) {
        let candidate = entry.join("valkey-server");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    bail!("failed to locate valkey-server on PATH; set VALKEY_SERVER_BIN explicitly if needed")
}

fn read_log_excerpt(path: &Path) -> Option<String> {
    let rendered = std::fs::read_to_string(path).ok()?;
    let trimmed = rendered.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lines = trimmed
        .lines()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" | ");
    Some(lines)
}
