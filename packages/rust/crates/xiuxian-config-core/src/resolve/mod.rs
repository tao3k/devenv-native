//! Layered TOML resolution, import expansion, and precedence helpers.

mod discover;
mod env;
mod imports;
mod io;
mod load;
mod merge;
mod namespace;
mod precedence;

pub use imports::{
    load_toml_value_with_imports, load_toml_value_with_imports_and_paths, merge_toml_values,
};
pub use load::{
    resolve_and_load, resolve_and_load_with_paths, resolve_and_merge_toml,
    resolve_and_merge_toml_with_paths,
};
pub use precedence::{
    NamedScalarValue, first_non_empty_lookup, first_non_empty_named_lookup, lookup_bool_flag,
    lookup_parsed, lookup_positive_parsed, parse_bool_flag, parse_positive, parse_trimmed,
    toml_first_env_parsed, toml_first_env_string, toml_first_named_string, trimmed_non_empty,
};
