//! Recursive TOML merge semantics shared by config loaders and import helpers.

use crate::ArrayMergeStrategy;

pub(super) fn merge_values(
    dst: &mut toml::Value,
    src: toml::Value,
    array_strategy: ArrayMergeStrategy,
) {
    match (dst, src) {
        (toml::Value::Table(dst_table), toml::Value::Table(src_table)) => {
            for (key, src_value) in src_table {
                if let Some(dst_value) = dst_table.get_mut(&key) {
                    merge_values(dst_value, src_value, array_strategy);
                } else {
                    dst_table.insert(key, src_value);
                }
            }
        }
        (toml::Value::Array(dst_array), toml::Value::Array(src_array))
            if matches!(array_strategy, ArrayMergeStrategy::Append) =>
        {
            dst_array.extend(src_array);
        }
        (dst_value, src_value) => {
            *dst_value = src_value;
        }
    }
}
