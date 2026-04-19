//! Parser-owned semantic-check grammar helpers.

mod api;
mod types;

#[cfg(any(test, feature = "zhenfa-router"))]
pub(crate) use self::api::{
    extract_function_args, extract_hash_references, extract_id_references, generate_suggested_id,
    validate_contract,
};
#[cfg(any(test, feature = "zhenfa-router"))]
pub use self::types::HashReference;

#[cfg(test)]
#[path = "../../../tests/unit/parsers/semantic_check.rs"]
mod tests;
