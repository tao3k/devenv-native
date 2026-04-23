mod api;
mod decode;
mod end;
mod root;
mod start;

pub(crate) use api::{
    append_capture_reference, append_capture_text, handle_end_tag, handle_start_tag, local_name,
    validate_dmn_root_start_tag,
};
