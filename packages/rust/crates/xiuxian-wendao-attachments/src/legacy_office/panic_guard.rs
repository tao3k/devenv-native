use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) fn run<T>(
    label: &str,
    path: &std::path::Path,
    parser: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(parser)).map_err(|payload| {
        format!(
            "parse legacy {label} `{}` panicked: {}",
            path.display(),
            panic_message(payload.as_ref())
        )
    })?
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "unknown panic payload".to_string()
}
