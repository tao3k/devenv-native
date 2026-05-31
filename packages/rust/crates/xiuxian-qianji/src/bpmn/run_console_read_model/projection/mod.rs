//! Deterministic rows for the qianji run-console read model.

mod contract;
mod elements;
mod event_text;
mod events;
mod metadata;
mod stream;

pub use contract::{
    QIANJI_CONTROL_RUN_STREAM_SCHEMA_VERSION, QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE,
    QIANJI_RUN_CONSOLE_EVENT_ROUTE, QIANJI_RUN_CONSOLE_SCHEMA_VERSION, QianjiControlRunStreamKind,
    QianjiControlRunStreamRow, QianjiControlRunStreamSource, QianjiRunConsoleElementState,
};
pub(crate) use contract::{
    QianjiRunConsoleElementProjection, QianjiRunConsoleElementStateRow, QianjiRunConsoleEventRow,
};
pub(crate) use elements::{
    qianji_run_console_element_projections, qianji_run_console_element_state_rows,
};
pub(crate) use events::qianji_run_console_event_rows;
pub use stream::qianji_control_run_stream_rows;
