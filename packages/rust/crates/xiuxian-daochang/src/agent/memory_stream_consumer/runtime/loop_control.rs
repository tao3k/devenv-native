use tokio::time::sleep;

use super::read_error::classify_stream_read_error;
use super::read_error_logging::{
    log_missing_consumer_group_recovery_attempt, log_missing_consumer_group_recovery_failure,
    log_stream_read_reconnect,
};
use crate::agent::memory_stream_consumer::processing::process_stream_events;
use crate::agent::memory_stream_consumer::stream::{
    connect_stream_consumer, ensure_consumer_group, ensure_consumer_group_before_read,
    open_stream_consumer_client, read_stream_events, stream_consumer_connection_config,
    stream_consumer_response_timeout_ms,
};
use crate::agent::memory_stream_consumer::types::{
    MemoryStreamConsumerRuntimeConfig, MemoryStreamEvent, StreamReadErrorKind,
    bump_failure_streak_and_backoff, reconnect_backoff_ms,
};

pub(in crate::agent::memory_stream_consumer) async fn run_consumer_loop(
    config: MemoryStreamConsumerRuntimeConfig,
) {
    let Some(client) = open_stream_consumer_client(&config) else {
        return;
    };
    let connection_config = stream_consumer_connection_config(config.stream_consumer_block_ms);
    let response_timeout_ms = stream_consumer_response_timeout_ms(config.stream_consumer_block_ms);

    let mut connect_failure_streak = 0_u32;
    let mut ensure_group_failure_streak = 0_u32;
    let mut read_failure_streak = 0_u32;
    loop {
        let Some(mut connection) = connect_stream_consumer(
            &client,
            &connection_config,
            &config,
            response_timeout_ms,
            &mut connect_failure_streak,
        )
        .await
        else {
            continue;
        };

        if !ensure_consumer_group_before_read(
            &mut connection,
            &config,
            &mut ensure_group_failure_streak,
        )
        .await
        {
            continue;
        }

        consume_stream_until_reconnect(&mut connection, &config, &mut read_failure_streak).await;
        sleep_reconnect_backoff(read_failure_streak).await;
    }
}

async fn consume_stream_until_reconnect(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    read_failure_streak: &mut u32,
) {
    let mut read_pending = true;
    loop {
        let stream_id = if read_pending { "0" } else { ">" };
        match read_stream_events(connection, config, stream_id).await {
            Ok(events) => {
                handle_stream_read_success(
                    connection,
                    config,
                    events,
                    &mut read_pending,
                    read_failure_streak,
                )
                .await;
            }
            Err(error) => {
                if handle_stream_read_error(
                    connection,
                    config,
                    stream_id,
                    error,
                    &mut read_pending,
                    read_failure_streak,
                )
                .await
                {
                    break;
                }
            }
        }
    }
}

async fn handle_stream_read_success(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    events: Vec<MemoryStreamEvent>,
    read_pending: &mut bool,
    read_failure_streak: &mut u32,
) {
    if events.is_empty() {
        if *read_pending {
            *read_pending = false;
        } else {
            *read_failure_streak = 0;
        }
        return;
    }

    let ack_failed = process_stream_events(connection, config, events, read_failure_streak).await;
    if !ack_failed {
        *read_failure_streak = 0;
    }
}

async fn handle_stream_read_error(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    stream_id: &str,
    error: anyhow::Error,
    read_pending: &mut bool,
    read_failure_streak: &mut u32,
) -> bool {
    let retry_backoff_ms = bump_failure_streak_and_backoff(read_failure_streak);
    let error_kind = classify_stream_read_error(&error);
    match error_kind {
        StreamReadErrorKind::MissingConsumerGroup => {
            handle_missing_consumer_group_error(
                connection,
                config,
                stream_id,
                &error,
                retry_backoff_ms,
                read_pending,
                *read_failure_streak,
            )
            .await
        }
        StreamReadErrorKind::Transport | StreamReadErrorKind::Other => {
            let error_kind_str = match error_kind {
                StreamReadErrorKind::MissingConsumerGroup => "missing_consumer_group",
                StreamReadErrorKind::Transport => "transport",
                StreamReadErrorKind::Other => "other",
            };
            log_stream_read_reconnect(
                config,
                stream_id,
                error_kind_str,
                &error,
                retry_backoff_ms,
                *read_failure_streak,
            );
            true
        }
    }
}

async fn handle_missing_consumer_group_error(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    stream_id: &str,
    error: &anyhow::Error,
    retry_backoff_ms: u64,
    read_pending: &mut bool,
    failure_streak: u32,
) -> bool {
    log_missing_consumer_group_recovery_attempt(
        config,
        stream_id,
        error,
        retry_backoff_ms,
        failure_streak,
    );

    match ensure_consumer_group(connection, config).await {
        Ok(()) => {
            *read_pending = true;
            sleep(std::time::Duration::from_millis(retry_backoff_ms)).await;
            false
        }
        Err(ensure_error) => {
            log_missing_consumer_group_recovery_failure(
                config,
                stream_id,
                error,
                &ensure_error,
                retry_backoff_ms,
                failure_streak,
            );
            true
        }
    }
}

async fn sleep_reconnect_backoff(read_failure_streak: u32) {
    let reconnect_backoff_ms = reconnect_backoff_ms(read_failure_streak);
    sleep(std::time::Duration::from_millis(reconnect_backoff_ms)).await;
}
