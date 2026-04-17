//! Session gate concurrency tests for shared and independent session keys.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::unit::live_gates::resolve_enabled_live_valkey_url;
use tokio::sync::oneshot;
use xiuxian_daochang::SessionGate;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn same_session_is_serialized() {
    let gate = SessionGate::default();
    let first_guard_result = gate.acquire("telegram:-100:888").await;
    let first_guard = match first_guard_result {
        Ok(guard) => guard,
        Err(error) => panic!("first session lock should succeed: {error}"),
    };

    let gate_for_second = gate.clone();
    let (entered_tx, entered_rx) = oneshot::channel::<()>();
    let blocked = Arc::new(AtomicBool::new(false));
    let blocked_for_task = Arc::clone(&blocked);

    let second = tokio::spawn(async move {
        blocked_for_task.store(true, Ordering::SeqCst);
        let second_guard_result = gate_for_second.acquire("telegram:-100:888").await;
        match second_guard_result {
            Ok(_second_guard) => {
                let _ = entered_tx.send(());
            }
            Err(error) => panic!("second session lock should succeed: {error}"),
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        blocked.load(Ordering::SeqCst),
        "second task should be waiting on the same session lock"
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), entered_rx)
            .await
            .is_err(),
        "second task should not enter before first lock is dropped"
    );

    drop(first_guard);
    if let Err(error) = second.await {
        panic!("second task should finish: {error}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn different_sessions_can_run_in_parallel() {
    let gate = SessionGate::default();
    let first_guard_result = gate.acquire("telegram:-100:888").await;
    let first_guard = match first_guard_result {
        Ok(guard) => guard,
        Err(error) => panic!("first session lock should succeed: {error}"),
    };

    let gate_for_other = gate.clone();
    let (entered_tx, entered_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let other_guard_result = gate_for_other.acquire("telegram:-101:888").await;
        match other_guard_result {
            Ok(_other_guard) => {
                let _ = entered_tx.send(());
            }
            Err(error) => panic!("other session lock should succeed: {error}"),
        }
    });

    let timeout_result = tokio::time::timeout(Duration::from_millis(200), entered_rx).await;
    if let Err(error) = timeout_result {
        panic!("other session should not be blocked: {error}");
    }

    drop(first_guard);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_entry_is_cleaned_after_last_guard_drops() {
    let gate = SessionGate::default();
    assert_eq!(gate.active_sessions(), 0);
    {
        let guard_result = gate.acquire("telegram:-100:888").await;
        let _guard = match guard_result {
            Ok(guard) => guard,
            Err(error) => panic!("session lock should succeed: {error}"),
        };
        assert_eq!(gate.active_sessions(), 1);
    }
    assert_eq!(gate.active_sessions(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn waiting_tasks_keep_session_entry_alive() {
    let gate = SessionGate::default();
    let first_guard_result = gate.acquire("telegram:-100:888").await;
    let first_guard = match first_guard_result {
        Ok(guard) => guard,
        Err(error) => panic!("first session lock should succeed: {error}"),
    };

    let gate_for_second = gate.clone();
    let (entered_second_tx, entered_second_rx) = oneshot::channel::<()>();
    let (release_second_tx, release_second_rx) = oneshot::channel::<()>();
    let second = tokio::spawn(async move {
        let second_guard_result = gate_for_second.acquire("telegram:-100:888").await;
        match second_guard_result {
            Ok(_second_guard) => {
                let _ = entered_second_tx.send(());
                let _ = release_second_rx.await;
            }
            Err(error) => panic!("second session lock should succeed: {error}"),
        }
    });

    tokio::time::sleep(Duration::from_millis(40)).await;
    assert_eq!(
        gate.active_sessions(),
        1,
        "entry should stay tracked while same-session task is waiting"
    );

    drop(first_guard);
    let entered_second_result =
        tokio::time::timeout(Duration::from_millis(200), entered_second_rx).await;
    if let Err(error) = entered_second_result {
        panic!("second task should enter after first guard drops: {error}");
    }

    let gate_for_third = gate.clone();
    let (entered_third_tx, entered_third_rx) = oneshot::channel::<()>();
    let third = tokio::spawn(async move {
        let third_guard_result = gate_for_third.acquire("telegram:-100:888").await;
        match third_guard_result {
            Ok(_third_guard) => {
                let _ = entered_third_tx.send(());
            }
            Err(error) => panic!("third session lock should succeed: {error}"),
        }
    });

    assert!(
        tokio::time::timeout(Duration::from_millis(60), entered_third_rx)
            .await
            .is_err(),
        "third task should still wait while second guard is held"
    );

    let _ = release_second_tx.send(());
    if let Err(error) = second.await {
        panic!("second task should finish: {error}");
    }
    if let Err(error) = third.await {
        panic!("third task should finish: {error}");
    }
    assert_eq!(gate.active_sessions(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn distributed_same_session_is_serialized_across_gate_instances() -> anyhow::Result<()> {
    let Some(valkey_url) = resolve_enabled_live_valkey_url("live session gate test") else {
        return Ok(());
    };
    let prefix = format!(
        "xiuxian-daochang:test:session-gate:same-session:{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_micros()
    );
    let gate_a =
        SessionGate::new_with_valkey_for_test(valkey_url.as_str(), prefix.as_str(), 30, Some(5))?;
    let gate_b =
        SessionGate::new_with_valkey_for_test(valkey_url.as_str(), prefix.as_str(), 30, Some(5))?;

    let first_guard = gate_a.acquire("telegram:-100:888").await?;
    let (entered_tx, entered_rx) = oneshot::channel::<()>();
    let second = tokio::spawn(async move {
        let second_guard_result = gate_b.acquire("telegram:-100:888").await;
        match second_guard_result {
            Ok(_second_guard) => {
                let _ = entered_tx.send(());
            }
            Err(error) => panic!("distributed lock should eventually succeed: {error}"),
        }
    });

    assert!(
        tokio::time::timeout(Duration::from_millis(200), entered_rx)
            .await
            .is_err(),
        "same session across gate instances should be serialized by distributed lease lock"
    );

    drop(first_guard);
    if let Err(error) = second.await {
        return Err(anyhow::anyhow!("second task should finish: {error}"));
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn distributed_different_sessions_run_in_parallel_across_gate_instances() -> anyhow::Result<()>
{
    let Some(valkey_url) = resolve_enabled_live_valkey_url("live session gate test") else {
        return Ok(());
    };
    let prefix = format!(
        "xiuxian-daochang:test:session-gate:parallel-session:{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_micros()
    );
    let gate_a =
        SessionGate::new_with_valkey_for_test(valkey_url.as_str(), prefix.as_str(), 30, Some(5))?;
    let gate_b =
        SessionGate::new_with_valkey_for_test(valkey_url.as_str(), prefix.as_str(), 30, Some(5))?;

    let _first_guard = gate_a.acquire("telegram:-100:888").await?;
    let (entered_tx, entered_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let second_guard_result = gate_b.acquire("telegram:-101:888").await;
        match second_guard_result {
            Ok(_second_guard) => {
                let _ = entered_tx.send(());
            }
            Err(error) => panic!("different sessions should not block each other: {error}"),
        }
    });

    let timeout_result = tokio::time::timeout(Duration::from_millis(300), entered_rx).await;
    if let Err(error) = timeout_result {
        return Err(anyhow::anyhow!(
            "different sessions across gate instances should execute in parallel: {error}"
        ));
    }
    Ok(())
}
