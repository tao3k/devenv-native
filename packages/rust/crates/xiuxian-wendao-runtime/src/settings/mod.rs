//! Project settings parsing and access helpers for runtime configuration.

mod access;
mod dirs;
mod parse;

pub use access::{get_setting_bool, get_setting_string, get_setting_string_list};
pub use dirs::{dedup_dirs, normalize_relative_dir};
pub use parse::{
    first_non_empty, parse_bool, parse_positive_f64, parse_positive_u64, parse_positive_usize,
};
