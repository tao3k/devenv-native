//! Qianji run-console read-model contract.

mod projection;
#[cfg(feature = "duckdb")]
mod schema;

pub use projection::{
    QIANJI_CONTROL_RUN_STREAM_SCHEMA_VERSION, QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE,
    QIANJI_RUN_CONSOLE_EVENT_ROUTE, QIANJI_RUN_CONSOLE_SCHEMA_VERSION, QianjiControlRunStreamRow,
    QianjiControlRunStreamSource, QianjiRunConsoleElementState, qianji_control_run_stream_rows,
};
pub(crate) use projection::{
    QianjiRunConsoleElementProjection, qianji_run_console_element_projections,
};
#[cfg(feature = "duckdb")]
pub use schema::{
    QianjiRunConsoleArrowReadModel, qianji_run_console_arrow_read_model,
    qianji_run_console_element_state_arrow_contract, qianji_run_console_element_state_arrow_schema,
    qianji_run_console_event_arrow_contract, qianji_run_console_event_arrow_schema,
};
