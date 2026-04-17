use super::support::{
    assert_telegram_group_merge, load_runtime_settings_from_paths, new_temp_settings_paths,
    write_file,
};

#[test]
fn merge_telegram_group_policy_overrides_deeply() {
    let (_tmp, system, user) = new_temp_settings_paths();

    write_file(
        &system,
        r#"
[telegram]
group_policy = "allowlist"
group_allow_from = "ops"
session_admin_persist = true
session_partition_persist = true
require_mention = true

[telegram.groups."*"]
require_mention = true

[telegram.groups."*".admin_users]
users = ["9090"]

[telegram.groups."*".topics."42"]
enabled = false

[telegram.groups."-100"]
group_policy = "disabled"

[telegram.groups."-100".allow_from]
users = ["root"]

[telegram.groups."-100".admin_users]
users = ["3001"]

[telegram.groups."-100".topics."10".allow_from]
users = ["ops1"]

[telegram.groups."-100".topics."10".admin_users]
users = ["7001"]
"#,
    );

    write_file(
        &user,
        r#"
[telegram]
group_policy = "open"
session_admin_persist = false
session_partition_persist = false
require_mention = false

[telegram.groups."-100".allow_from]
users = ["admin2"]

[telegram.groups."-100".admin_users]
users = ["3002"]

[telegram.groups."-100".topics."10"]
require_mention = true

[telegram.groups."-100".topics."10".admin_users]
users = ["7002"]

[telegram.groups."-100".topics."11"]
enabled = true

[telegram.groups."-100".topics."11".admin_users]
users = ["8001"]

[telegram.groups."-200"]
enabled = true

[telegram.groups."-200".admin_users]
users = ["4001"]
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_telegram_group_merge(&merged);
}
