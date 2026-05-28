use anyhow::{anyhow, bail};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QianjiServerCommand {
    Serve(QianjiServerServeCommand),
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QianjiServerServeCommand {
    pub(crate) bind_addr: Option<SocketAddr>,
    pub(crate) flight_bind_addr: Option<SocketAddr>,
    pub(crate) valkey_url: Option<String>,
    pub(crate) require_valkey_ready: Option<bool>,
    pub(crate) flowhub_root: Option<PathBuf>,
    pub(crate) control_ledger_path: Option<PathBuf>,
}

pub(crate) fn parse_qianji_server_args<I, S>(args: I) -> anyhow::Result<QianjiServerCommand>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut bind_addr = None;
    let mut flight_bind_addr = None;
    let mut valkey_url = None;
    let mut require_valkey_ready = None;
    let mut flowhub_root = None;
    let mut control_ledger_path = None;
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            return Ok(QianjiServerCommand::Help);
        }

        if arg == "--bind" {
            let value = args
                .next()
                .ok_or_else(|| anyhow!("missing value for --bind\n\n{}", qianji_server_usage()))?;
            bind_addr = Some(parse_bind_addr(&value)?);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--bind=") {
            bind_addr = Some(parse_bind_addr(value)?);
            continue;
        }

        if arg == "--flight-bind" {
            let value = args.next().ok_or_else(|| {
                anyhow!(
                    "missing value for --flight-bind\n\n{}",
                    qianji_server_usage()
                )
            })?;
            flight_bind_addr = Some(parse_flight_bind_addr(&value)?);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--flight-bind=") {
            flight_bind_addr = Some(parse_flight_bind_addr(value)?);
            continue;
        }

        if arg == "--valkey-url" {
            let value = args.next().ok_or_else(|| {
                anyhow!(
                    "missing value for --valkey-url\n\n{}",
                    qianji_server_usage()
                )
            })?;
            valkey_url = Some(parse_valkey_url(&value)?);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--valkey-url=") {
            valkey_url = Some(parse_valkey_url(value)?);
            continue;
        }

        if arg == "--flowhub-root" {
            let value = args.next().ok_or_else(|| {
                anyhow!(
                    "missing value for --flowhub-root\n\n{}",
                    qianji_server_usage()
                )
            })?;
            flowhub_root = Some(parse_flowhub_root(&value)?);
            continue;
        }

        if arg == "--control-ledger" {
            let value = args.next().ok_or_else(|| {
                anyhow!(
                    "missing value for --control-ledger\n\n{}",
                    qianji_server_usage()
                )
            })?;
            control_ledger_path = Some(parse_control_ledger_path(&value)?);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--control-ledger=") {
            control_ledger_path = Some(parse_control_ledger_path(value)?);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--flowhub-root=") {
            flowhub_root = Some(parse_flowhub_root(value)?);
            continue;
        }

        if arg == "--require-valkey-ready" {
            require_valkey_ready = Some(true);
            continue;
        }

        if arg == "--no-require-valkey-ready" {
            require_valkey_ready = Some(false);
            continue;
        }

        bail!(
            "unsupported qianji-server argument `{arg}`\n\n{}",
            qianji_server_usage()
        );
    }

    Ok(QianjiServerCommand::Serve(QianjiServerServeCommand {
        bind_addr,
        flight_bind_addr,
        valkey_url,
        require_valkey_ready,
        flowhub_root,
        control_ledger_path,
    }))
}

pub(crate) fn qianji_server_usage() -> &'static str {
    "Usage: qianji-server [--bind <addr>] [--flight-bind <addr>] [--valkey-url <url>] [--flowhub-root <path>] [--control-ledger <path>] [--require-valkey-ready|--no-require-valkey-ready]\n\nStarts the Qianji BPMN HTTP service and Arrow Flight run-console data-plane listener. When --bind is omitted, [server].bind_addr from qianji.toml is used. When --flight-bind is omitted, [server].flight_bind_addr from qianji.toml is used. HTTP checkpoint defaults are Valkey-only. --control-ledger overrides the default DuckDB control ledger path."
}

fn parse_bind_addr(value: &str) -> anyhow::Result<SocketAddr> {
    value
        .parse()
        .map_err(|error| anyhow!("invalid --bind address `{value}`: {error}"))
}

fn parse_flight_bind_addr(value: &str) -> anyhow::Result<SocketAddr> {
    value
        .parse()
        .map_err(|error| anyhow!("invalid --flight-bind address `{value}`: {error}"))
}

fn parse_valkey_url(value: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("--valkey-url must not be empty");
    }
    Ok(value.to_string())
}

fn parse_flowhub_root(value: &str) -> anyhow::Result<PathBuf> {
    let value = value.trim();
    if value.is_empty() {
        bail!("--flowhub-root must not be empty");
    }
    Ok(PathBuf::from(value))
}

fn parse_control_ledger_path(value: &str) -> anyhow::Result<PathBuf> {
    let value = value.trim();
    if value.is_empty() {
        bail!("--control-ledger must not be empty");
    }
    Ok(PathBuf::from(value))
}
