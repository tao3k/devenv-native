use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::build_qianji_server_router;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use tokio::net::TcpListener;

pub(super) fn must_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

pub(super) fn must_err<T, E>(result: Result<T, E>, context: &str) -> String
where
    E: std::fmt::Display,
{
    match result {
        Ok(_) => panic!("{context}: expected error"),
        Err(error) => error.to_string(),
    }
}

pub(super) fn must_parse_addr(value: &str) -> SocketAddr {
    match value.parse() {
        Ok(addr) => addr,
        Err(error) => panic!("bind address should be valid: {error}"),
    }
}

pub(super) fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("fixture parent should be created: {error}"));
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("fixture file should write: {error}"));
}

pub(super) async fn spawn_qianji_server_router(command: QianjiServerServeCommand) -> String {
    let router = must_ok(
        build_qianji_server_router(&command),
        "qianji-server router should build",
    );
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|error| panic!("test server should bind: {error}"));
    let addr = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("test server local address should resolve: {error}"));
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            panic!("test qianji-server router should serve: {error}");
        }
    });
    format!("http://{addr}")
}
