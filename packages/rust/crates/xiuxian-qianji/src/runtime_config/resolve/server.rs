use crate::runtime_config::constants::{
    DEFAULT_SERVER_BIND_ADDR, DEFAULT_SERVER_REQUIRE_VALKEY_READY,
};
use crate::runtime_config::env_vars::{env_var_or_override, parse_bool_env_override};
use crate::runtime_config::model::{QianjiRuntimeEnv, QianjiRuntimeServerConfig};
use crate::runtime_config::toml_config::QianjiTomlServer;
use xiuxian_macros::string_first_non_empty;

pub(super) fn resolve_qianji_runtime_server(
    file_server: &QianjiTomlServer,
    runtime_env: &QianjiRuntimeEnv,
) -> QianjiRuntimeServerConfig {
    let bind_addr = string_first_non_empty!(
        runtime_env.qianji_server_bind_addr.as_deref(),
        file_server.bind_addr.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_SERVER_BIND_ADDR").as_deref(),
        Some(DEFAULT_SERVER_BIND_ADDR),
    );
    let require_valkey_ready = runtime_env
        .qianji_server_require_valkey_ready
        .or(file_server.require_valkey_ready)
        .or_else(|| parse_bool_env_override(runtime_env, "QIANJI_SERVER_REQUIRE_VALKEY_READY"))
        .unwrap_or(DEFAULT_SERVER_REQUIRE_VALKEY_READY);

    QianjiRuntimeServerConfig {
        bind_addr,
        require_valkey_ready,
    }
}
