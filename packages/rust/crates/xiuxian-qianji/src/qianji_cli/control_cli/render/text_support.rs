use std::fmt;

use xiuxian_qianji_control::RecoveryItemScope;

pub(crate) fn push_fmt(output: &mut String, args: fmt::Arguments<'_>) {
    if fmt::write(output, args).is_err() {
        unreachable!("writing to a String cannot fail");
    }
}

pub(crate) fn activity_scope_label(scope: &RecoveryItemScope) -> String {
    match scope {
        RecoveryItemScope::Run => "run".to_owned(),
        RecoveryItemScope::Step { step_id } => format!("step:{}", step_id.as_str()),
    }
}

pub(crate) fn serde_status<T>(status: &T) -> String
where
    T: serde::Serialize,
{
    serde_json::to_string(status)
        .unwrap_or_else(|_| "\"unknown\"".to_string())
        .trim_matches('"')
        .to_string()
}
