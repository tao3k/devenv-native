use std::io;

use crate::qianji_cli::invalid_input;

pub(super) fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
