use super::{DEFAULT_PULSE_CHANNEL, PulseEmitter, SwarmEvent};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

const DEFAULT_EVENT_QUEUE_CAPACITY: usize = 2048;
const RECONNECT_BACKOFF_MS: u64 = 200;

/// Non-blocking pulse emitter backed by Valkey Pub/Sub.
///
/// Emission path is always asynchronous:
/// - caller serializes + enqueue (`try_send`)
/// - dedicated background task publishes to Valkey
#[derive(Debug)]
pub struct ValkeyPulseEmitter {
    channel: Arc<str>,
    queue_tx: mpsc::Sender<Arc<str>>,
    sample_counter: AtomicU64,
    dropped_events: AtomicU64,
}

impl ValkeyPulseEmitter {
    /// Creates a pulse emitter that publishes to [`DEFAULT_PULSE_CHANNEL`].
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self::with_channel(redis_url, DEFAULT_PULSE_CHANNEL.to_string())
    }

    /// Creates a pulse emitter with explicit channel name.
    #[must_use]
    pub fn with_channel(redis_url: String, channel: String) -> Self {
        let (queue_tx, queue_rx) = mpsc::channel(DEFAULT_EVENT_QUEUE_CAPACITY);
        let redis_url = Arc::<str>::from(redis_url);
        let channel = Arc::<str>::from(channel);
        std::mem::drop(tokio::spawn(Self::run_publish_loop(
            redis_url,
            Arc::clone(&channel),
            queue_rx,
        )));
        Self {
            channel,
            queue_tx,
            sample_counter: AtomicU64::new(0),
            dropped_events: AtomicU64::new(0),
        }
    }

    /// Returns total number of events dropped by local backpressure sampling/full queue.
    #[must_use]
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::Relaxed)
    }

    /// Returns target Pub/Sub channel name.
    #[must_use]
    pub fn channel(&self) -> &str {
        &self.channel
    }

    async fn run_publish_loop(
        redis_url: Arc<str>,
        channel: Arc<str>,
        mut queue_rx: mpsc::Receiver<Arc<str>>,
    ) {
        let mut connection: Option<redis::aio::MultiplexedConnection> = None;
        while let Some(payload) = queue_rx.recv().await {
            if let Err(error) = publish_payload(
                redis_url.as_ref(),
                channel.as_ref(),
                payload.as_ref(),
                &mut connection,
            )
            .await
            {
                log::warn!(
                    "swarm pulse publish failed on channel '{}': {error}",
                    channel.as_ref()
                );
            }
        }
    }

    fn should_sample_event(&self) -> bool {
        let sample_rate = self.current_sample_rate();
        if sample_rate <= 1 {
            return false;
        }
        let slot = self.sample_counter.fetch_add(1, Ordering::Relaxed);
        !slot.is_multiple_of(sample_rate)
    }

    fn current_sample_rate(&self) -> u64 {
        let free_capacity = self.queue_tx.capacity();
        if free_capacity < 64 {
            8
        } else if free_capacity < 128 {
            4
        } else if free_capacity < 256 {
            2
        } else {
            1
        }
    }
}

#[async_trait]
impl PulseEmitter for ValkeyPulseEmitter {
    async fn emit_pulse(&self, event: SwarmEvent) -> Result<(), String> {
        if self.should_sample_event() {
            return Ok(());
        }
        let payload = serde_json::to_string(&event).map_err(|error| error.to_string())?;
        match self.queue_tx.try_send(Arc::<str>::from(payload)) {
            Ok(()) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                self.dropped_events.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                Err("swarm pulse emitter is closed".to_string())
            }
        }
    }
}

async fn publish_payload(
    redis_url: &str,
    channel: &str,
    payload: &str,
    connection: &mut Option<redis::aio::MultiplexedConnection>,
) -> Result<(), String> {
    if connection.is_none() {
        *connection = Some(connect_valkey(redis_url).await?);
    }

    if try_publish_once(channel, payload, connection).await.is_ok() {
        return Ok(());
    }

    *connection = None;
    sleep(Duration::from_millis(RECONNECT_BACKOFF_MS)).await;
    *connection = Some(connect_valkey(redis_url).await?);
    try_publish_once(channel, payload, connection).await
}

async fn connect_valkey(redis_url: &str) -> Result<redis::aio::MultiplexedConnection, String> {
    let client = redis::Client::open(redis_url).map_err(|error| error.to_string())?;
    client
        .get_multiplexed_async_connection()
        .await
        .map_err(|error| error.to_string())
}

async fn try_publish_once(
    channel: &str,
    payload: &str,
    connection: &mut Option<redis::aio::MultiplexedConnection>,
) -> Result<(), String> {
    let Some(connection) = connection.as_mut() else {
        return Err("missing valkey connection".to_string());
    };
    let mut command = redis::cmd("PUBLISH");
    command.arg(channel).arg(payload);
    command
        .query_async::<i64>(connection)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}
