use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, InMemoryControlLedger, RecoveryItemScope, RunId,
    SignalName, SignalReceiveJournalRecord, SignalRecord, TimerId, TimerRecord, TimerStatus,
    VersionKey, VersionPin, record_signal_received,
};

use crate::control::support::artifact_ref;

#[test]
fn in_memory_ledger_replays_signal_timer_and_version_events() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-signal-timer-version")?;
    let timer_id = TimerId::new("approval-timeout")?;
    let version_key = VersionKey::new("flowhub_version")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "wait for approval".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        2,
        ControlEventKind::VersionPinned {
            pin: VersionPin {
                version_key: version_key.clone(),
                version: "2026-05-17.flowhub".to_owned(),
                content_hash: Some("sha256:flowhub".to_owned()),
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    record_signal_received(
        &ledger,
        SignalReceiveJournalRecord::new(
            run_id.clone(),
            RecoveryItemScope::run(),
            SignalRecord {
                signal_name: SignalName::new("human.approval")?,
                payload_ref: Some(artifact_ref("artifact-human-approval")?),
                payload_hash: Some("sha256:approval".to_owned()),
                metadata: serde_json::Value::Null,
            },
            3,
        ),
    )?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        4,
        ControlEventKind::TimerScheduled {
            timer: TimerRecord {
                timer_id: timer_id.clone(),
                fire_at_ms: 10_000,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        10_000,
        ControlEventKind::TimerFired {
            timer_id: timer_id.clone(),
        },
    ))?;

    let view = ledger.load_run_view(&run_id)?;
    let timer = view
        .timers
        .get(&timer_id)
        .ok_or_else(|| io::Error::other("missing replayed timer"))?;

    assert_eq!(view.signals.len(), 1);
    assert_eq!(
        view.signals[0].signal_name,
        SignalName::new("human.approval")?
    );
    assert_eq!(
        view.version_pins
            .get(&version_key)
            .map(|pin| pin.content_hash.as_deref()),
        Some(Some("sha256:flowhub"))
    );
    assert_eq!(timer.status, TimerStatus::Fired);
    assert_eq!(timer.fired_at_ms, Some(10_000));

    Ok(())
}
