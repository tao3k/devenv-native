pub(super) use xiuxian_daochang::{DiscordChannel, DiscordSessionPartition};

macro_rules! parse_message {
    ($channel:expr, $event:expr, $context:literal) => {{
        match $channel.parse_gateway_message($event) {
            Some(parsed) => parsed,
            None => panic!($context),
        }
    }};
}

pub(super) use parse_message;

pub(super) fn discord_event(
    message_id: &str,
    content: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: Option<&str>,
) -> serde_json::Value {
    discord_event_with_roles(
        message_id,
        content,
        channel_id,
        guild_id,
        user_id,
        username,
        &[],
    )
}

pub(super) fn discord_event_with_roles(
    message_id: &str,
    content: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: Option<&str>,
    role_ids: &[&str],
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "id": message_id,
        "content": content,
        "channel_id": channel_id,
        "author": {
            "id": user_id
        }
    });
    if let Some(guild) = guild_id {
        payload["guild_id"] = serde_json::Value::String(guild.to_string());
    }
    if let Some(name) = username {
        payload["author"]["username"] = serde_json::Value::String(name.to_string());
    }
    if !role_ids.is_empty() {
        payload["member"] = serde_json::json!({
            "roles": role_ids,
        });
    }
    payload
}

pub(super) fn discord_event_with_mentions(
    message_id: &str,
    content: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: Option<&str>,
    mentioned_user_ids: &[&str],
) -> serde_json::Value {
    let mut payload = discord_event(message_id, content, channel_id, guild_id, user_id, username);
    if !mentioned_user_ids.is_empty() {
        payload["mentions"] = serde_json::Value::Array(
            mentioned_user_ids
                .iter()
                .map(|mentioned| serde_json::json!({ "id": mentioned }))
                .collect(),
        );
    }
    payload
}

pub(super) fn discord_event_reply_to(
    message_id: &str,
    content: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: Option<&str>,
    reply_author_id: &str,
) -> serde_json::Value {
    let mut payload = discord_event(message_id, content, channel_id, guild_id, user_id, username);
    payload["referenced_message"] = serde_json::json!({
        "author": {
            "id": reply_author_id,
        }
    });
    payload
}

pub(super) fn discord_slash_interaction_event(
    interaction_id: &str,
    command_name: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    username: &str,
    options: serde_json::Value,
    interaction_type: u8,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "id": interaction_id,
        "application_id": "5001",
        "type": interaction_type,
        "data": {
            "id": "6001",
            "name": command_name,
            "type": 1
        },
        "channel_id": channel_id,
        "token": "interaction-token",
        "version": 1,
        "locale": "en-US",
        "entitlements": [],
        "attachment_size_limit": 8_388_608,
        "user": {
            "id": user_id,
            "username": username
        }
    });
    if let Some(guild) = guild_id {
        payload["guild_id"] = serde_json::Value::String(guild.to_string());
        payload["guild_locale"] = serde_json::Value::String("en-US".to_string());
    }
    if !options.is_null() {
        payload["data"]["options"] = options;
    }
    payload
}
