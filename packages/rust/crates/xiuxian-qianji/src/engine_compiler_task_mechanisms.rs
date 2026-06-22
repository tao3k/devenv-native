//! Task-mechanism branch modules used by the Qianji compiler.

#[path = "engine/compiler/task_mechanisms/invocation.rs"]
mod invocation;
#[path = "engine/compiler/task_mechanisms/io_control.rs"]
mod io_control;
#[path = "engine/compiler/task_mechanisms/quality.rs"]
mod quality;
#[path = "engine/compiler/task_mechanisms/wendao_router.rs"]
mod wendao_router;
#[cfg(feature = "wendao-integration")]
#[path = "engine/compiler/task_mechanisms/wendao_sql.rs"]
mod wendao_sql;

pub(super) use invocation::{cli_call, http_call};
pub(super) use io_control::{command, suspend, write_file};
pub(super) use quality::{calibration, mock, security_scan};
pub(super) use wendao_router::router;
#[cfg(feature = "wendao-integration")]
pub(super) use wendao_router::{wendao_ingester, wendao_refresh};
#[cfg(feature = "wendao-integration")]
pub(super) use wendao_sql::{wendao_sql_discover, wendao_sql_execute, wendao_sql_validate};
