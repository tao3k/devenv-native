use super::*;

#[test]
fn observer_default_creation() {
    let observer = ArtifactObserver::default();
    assert!(observer.config().enabled);
}

#[test]
fn observer_should_handle_exit_event() {
    let observer = ArtifactObserver::default();
    let event = SwarmEvent::NodeTransition {
        session_id: Some("session-1".to_string()),
        agent_id: None,
        role_class: None,
        node_id: "TestNode".to_string(),
        phase: NodeTransitionPhase::Exiting,
        timestamp_ms: 1_700_000_000_000,
    };
    assert!(observer.should_handle_event(&event));
}

#[test]
fn observer_should_handle_failed_event() {
    let observer = ArtifactObserver::default();
    let event = SwarmEvent::NodeTransition {
        session_id: Some("session-1".to_string()),
        agent_id: None,
        role_class: None,
        node_id: "TestNode".to_string(),
        phase: NodeTransitionPhase::Failed,
        timestamp_ms: 1_700_000_000_000,
    };
    assert!(observer.should_handle_event(&event));
}

#[test]
fn observer_should_not_handle_entering_event() {
    let observer = ArtifactObserver::default();
    let event = SwarmEvent::NodeTransition {
        session_id: Some("session-1".to_string()),
        agent_id: None,
        role_class: None,
        node_id: "TestNode".to_string(),
        phase: NodeTransitionPhase::Entering,
        timestamp_ms: 1_700_000_000_000,
    };
    assert!(!observer.should_handle_event(&event));
}

#[test]
fn observer_disabled_ignores_events() {
    let config = ArtifactObserverConfig {
        enabled: false,
        ..Default::default()
    };
    let observer = ArtifactObserver::new(config, NoopWendaoIngestionSink);
    let event = SwarmEvent::NodeTransition {
        session_id: None,
        agent_id: None,
        role_class: None,
        node_id: "TestNode".to_string(),
        phase: NodeTransitionPhase::Exiting,
        timestamp_ms: 0,
    };
    assert!(!observer.should_handle_event(&event));
}

#[test]
fn observer_ignores_non_transition_events() {
    let observer = ArtifactObserver::default();
    let event = SwarmEvent::SwarmHeartbeat {
        session_id: None,
        cluster_id: None,
        agent_id: None,
        role_class: None,
        cpu_percent: None,
        memory_bytes: None,
        timestamp_ms: 0,
    };
    assert!(!observer.should_handle_event(&event));
}
