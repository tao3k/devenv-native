use anyhow::{anyhow, bail};
use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QianjiServerCommand {
    Serve(QianjiServerServeCommand),
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QianjiServerServeCommand {
    pub(crate) bind_addr: Option<SocketAddr>,
    pub(crate) valkey_url: Option<String>,
    pub(crate) require_valkey_ready: Option<bool>,
}

pub(crate) fn parse_qianji_server_args<I, S>(args: I) -> anyhow::Result<QianjiServerCommand>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut bind_addr = None;
    let mut valkey_url = None;
    let mut require_valkey_ready = None;
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
        valkey_url,
        require_valkey_ready,
    }))
}

pub(crate) fn qianji_server_usage() -> &'static str {
    "Usage: qianji-server [--bind <addr>] [--valkey-url <url>] [--require-valkey-ready|--no-require-valkey-ready]\n\nStarts the Qianji BPMN HTTP service. When --bind is omitted, [server].bind_addr from qianji.toml is used. HTTP checkpoint defaults are Valkey-only."
}

fn parse_bind_addr(value: &str) -> anyhow::Result<SocketAddr> {
    value
        .parse()
        .map_err(|error| anyhow!("invalid --bind address `{value}`: {error}"))
}

fn parse_valkey_url(value: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("--valkey-url must not be empty");
    }
    Ok(value.to_string())
}
