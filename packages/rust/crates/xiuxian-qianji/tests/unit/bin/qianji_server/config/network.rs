use super::support::{must_ok, must_parse_addr, server_command, write_qianji_server_config};
use crate::qianji_server_cli::run::{
    resolve_qianji_server_bind_addr_with_env, resolve_qianji_server_flight_bind_addr_with_env,
};
use crate::runtime_config::QianjiRuntimeEnv;

#[test]
fn qianji_server_bind_addr_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );

    let command = server_command();
    let bind_addr = must_ok(
        resolve_qianji_server_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "qianji-server bind address should resolve from qianji.toml",
    );

    assert_eq!(bind_addr, must_parse_addr("127.0.0.1:38131"));
}

#[test]
fn qianji_server_cli_bind_overrides_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );

    let mut command = server_command();
    command.bind_addr = Some(must_parse_addr("127.0.0.1:38132"));
    let bind_addr = must_ok(
        resolve_qianji_server_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "CLI bind should override qianji.toml",
    );

    assert_eq!(bind_addr, must_parse_addr("127.0.0.1:38132"));
}

#[test]
fn qianji_server_flight_bind_addr_defaults_from_runtime_config() {
    let command = server_command();
    let flight_bind_addr = must_ok(
        resolve_qianji_server_flight_bind_addr_with_env(&command, &QianjiRuntimeEnv::default()),
        "qianji-server Flight bind address should resolve by default",
    );

    assert_eq!(flight_bind_addr, Some(must_parse_addr("127.0.0.1:38131")));
}

#[test]
fn qianji_server_flight_bind_addr_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
flight_bind_addr = "127.0.0.1:38136"
"#,
    );

    let command = server_command();
    let flight_bind_addr = must_ok(
        resolve_qianji_server_flight_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "qianji-server Flight bind address should resolve from qianji.toml",
    );

    assert_eq!(flight_bind_addr, Some(must_parse_addr("127.0.0.1:38136")));
}

#[test]
fn qianji_server_cli_flight_bind_overrides_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
flight_bind_addr = "127.0.0.1:38136"
"#,
    );

    let mut command = server_command();
    command.flight_bind_addr = Some(must_parse_addr("127.0.0.1:38137"));
    let flight_bind_addr = must_ok(
        resolve_qianji_server_flight_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "CLI Flight bind should override qianji.toml",
    );

    assert_eq!(flight_bind_addr, Some(must_parse_addr("127.0.0.1:38137")));
}
