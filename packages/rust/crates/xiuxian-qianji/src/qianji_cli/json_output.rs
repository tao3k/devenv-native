use std::io;
use std::path::Path;

use serde::Serialize;

pub(crate) const QIANJI_CLI_JSON_SCHEMA_VERSION: u32 = 1;

pub(crate) struct CliJsonEnvelope<'a, T>
where
    T: Serialize,
{
    pub(crate) kind: &'a str,
    pub(crate) command: &'a str,
    pub(crate) domain: &'a str,
    pub(crate) path: &'a Path,
    pub(crate) source_id: &'a str,
    pub(crate) ok: bool,
    pub(crate) exit_code: i32,
    pub(crate) report: T,
    pub(crate) analysis: Option<serde_json::Value>,
}

pub(crate) fn render_cli_json<T>(envelope: CliJsonEnvelope<'_, T>) -> io::Result<String>
where
    T: Serialize,
{
    let mut value = serde_json::json!({
        "kind": envelope.kind,
        "schema_version": QIANJI_CLI_JSON_SCHEMA_VERSION,
        "command": envelope.command,
        "domain": envelope.domain,
        "ok": envelope.ok,
        "exit_code": envelope.exit_code,
        "path": envelope.path,
        "source": {
            "path": envelope.path,
            "source_id": envelope.source_id,
        },
        "report": envelope.report,
    });

    if let Some(analysis) = envelope.analysis {
        value["analysis"] = analysis;
    }

    serde_json::to_string_pretty(&value).map_err(|error| json_error(&error))
}

pub(crate) fn json_error(error: &serde_json::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("failed to render qianji CLI JSON: {error}"),
    )
}
