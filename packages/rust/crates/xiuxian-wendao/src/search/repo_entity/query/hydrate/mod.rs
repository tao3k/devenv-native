#[path = "columns.rs"]
mod columns;
#[path = "load.rs"]
mod load;
#[path = "parse.rs"]
mod parse;
#[path = "rows.rs"]
mod rows;

pub(crate) use columns::{
    engine_float64_column, engine_list_string_column, engine_list_string_values,
    engine_string_column, engine_uint32_column, hit_json_projection_columns, id_filter_expression,
    optional_engine_string_value, optional_engine_u32_value, typed_repo_entity_columns,
};
pub(crate) use load::{hydrate_repo_entity_hits, load_hydrated_rows_by_id};
pub(crate) use parse::{
    non_empty_vec, parse_attributes_map, parse_backlink_items, parse_import_kind, parse_symbol_kind,
};
pub(crate) use rows::{
    build_example_search_result, build_import_search_result, build_module_search_result,
    build_symbol_search_result,
};

#[cfg(test)]
#[path = "../../../../../tests/unit/search/repo_entity/query/hydrate/mod.rs"]
mod tests;
