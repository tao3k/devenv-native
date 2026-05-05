use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::runtime::Handle;

use crate::studio::search;
use crate::studio::symbol_index::state::{SymbolIndexCoordinator, timestamp_now};
use crate::studio::symbol_index::{SymbolIndexPhase, SymbolIndexStatus};
use crate::studio::types::UiProjectConfig;
use xiuxian_wendao::search::{
    LocalSymbolSearchError, SearchCorpusKind, SearchPlanePhase, SearchPlaneService,
};
use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;

const LOCAL_SYMBOL_RESTORE_POLL_INTERVAL: Duration = Duration::from_millis(25);

#[allow(clippy::too_many_lines)]
pub(crate) fn maybe_spawn_build(
    coordinator: &Arc<SymbolIndexCoordinator>,
    projects: Vec<UiProjectConfig>,
    index_cache: Arc<RwLock<Option<Arc<UnifiedSymbolIndex>>>>,
    fingerprint: String,
) {
    let _spawn_guard = coordinator
        .spawn_lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let current_fingerprint = coordinator
        .active_fingerprint
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let current_status = coordinator.status();
    let current_index = index_cache
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();

    if current_fingerprint.as_deref() == Some(fingerprint.as_str()) {
        if current_index.is_some() && matches!(current_status.phase, SymbolIndexPhase::Ready) {
            return;
        }
        if matches!(current_status.phase, SymbolIndexPhase::Indexing) {
            return;
        }
    }

    *coordinator
        .active_fingerprint
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(fingerprint.clone());
    coordinator
        .shutdown_requested
        .store(false, Ordering::SeqCst);
    *coordinator
        .status
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = SymbolIndexStatus {
        phase: SymbolIndexPhase::Indexing,
        last_error: None,
        updated_at: Some(timestamp_now()),
    };

    let project_root = coordinator.project_root.clone();
    let config_root = coordinator.config_root.clone();
    let search_plane = coordinator.search_plane.clone();
    let active_fingerprint = Arc::clone(&coordinator.active_fingerprint);
    let status = Arc::clone(&coordinator.status);
    let shutdown_requested = Arc::clone(&coordinator.shutdown_requested);

    if let Ok(handle) = Handle::try_current() {
        let build_task = handle.spawn(async move {
            let restore = try_restore_symbol_index(
                &search_plane,
                fingerprint.as_str(),
                Arc::clone(&active_fingerprint),
                Arc::clone(&shutdown_requested),
            )
            .await;
            if should_stop_build(
                &active_fingerprint,
                &shutdown_requested,
                fingerprint.as_str(),
            ) {
                return;
            }

            match restore {
                Ok(Some((index, phase))) => {
                    *index_cache
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(index));
                    *status
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = SymbolIndexStatus {
                        phase,
                        last_error: None,
                        updated_at: Some(timestamp_now()),
                    };

                    if matches!(phase, SymbolIndexPhase::Ready) {
                        return;
                    }

                    match wait_for_fresh_local_symbol_artifact(
                        &search_plane,
                        fingerprint.as_str(),
                        Arc::clone(&active_fingerprint),
                        Arc::clone(&shutdown_requested),
                    )
                    .await
                    {
                        Ok(Some(index)) => {
                            *index_cache
                                .write()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                                Some(Arc::new(index));
                            *status
                                .write()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                                SymbolIndexStatus {
                                    phase: SymbolIndexPhase::Ready,
                                    last_error: None,
                                    updated_at: Some(timestamp_now()),
                                };
                        }
                        Ok(None) => {}
                        Err(error) => mark_symbol_index_failed(&index_cache, &status, error),
                    }
                }
                Ok(None) => {
                    let build = tokio::task::spawn_blocking(move || {
                        search::build_symbol_index(
                            project_root.as_path(),
                            config_root.as_path(),
                            &projects,
                        )
                    })
                    .await;

                    if should_stop_build(
                        &active_fingerprint,
                        &shutdown_requested,
                        fingerprint.as_str(),
                    ) {
                        return;
                    }

                    match build {
                        Ok(index) => {
                            *index_cache
                                .write()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                                Some(Arc::new(index));
                            *status
                                .write()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                                SymbolIndexStatus {
                                    phase: SymbolIndexPhase::Ready,
                                    last_error: None,
                                    updated_at: Some(timestamp_now()),
                                };
                        }
                        Err(error) => mark_symbol_index_failed(
                            &index_cache,
                            &status,
                            format!("symbol index background task panicked: {error}"),
                        ),
                    }
                }
                Err(error) => mark_symbol_index_failed(&index_cache, &status, error),
            }
        });
        *coordinator
            .build_task
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(build_task);
    } else {
        *coordinator
            .status
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = SymbolIndexStatus {
            phase: SymbolIndexPhase::Failed,
            last_error: Some("Tokio runtime unavailable for symbol index build".to_string()),
            updated_at: Some(timestamp_now()),
        };
    }
}

async fn try_restore_symbol_index(
    search_plane: &SearchPlaneService,
    fingerprint: &str,
    active_fingerprint: Arc<RwLock<Option<String>>>,
    shutdown_requested: Arc<std::sync::atomic::AtomicBool>,
) -> Result<Option<(UnifiedSymbolIndex, SymbolIndexPhase)>, String> {
    match xiuxian_wendao::search::restore_local_symbol_hits(search_plane).await {
        Ok(hits) => {
            let phase = match current_local_symbol_phase(search_plane, fingerprint) {
                SearchPlanePhase::Indexing => SymbolIndexPhase::Indexing,
                _ => SymbolIndexPhase::Ready,
            };
            let hits = hits.into_iter().map(Into::into).collect::<Vec<_>>();
            Ok(Some((
                search::build_symbol_index_from_ast_hits(hits.as_slice()),
                phase,
            )))
        }
        Err(LocalSymbolSearchError::NotReady)
            if matches!(
                current_local_symbol_phase(search_plane, fingerprint),
                SearchPlanePhase::Indexing
            ) =>
        {
            wait_for_fresh_local_symbol_artifact(
                search_plane,
                fingerprint,
                active_fingerprint,
                shutdown_requested,
            )
            .await
            .map(|maybe_index| maybe_index.map(|index| (index, SymbolIndexPhase::Ready)))
        }
        Err(LocalSymbolSearchError::NotReady) => Ok(None),
        Err(error) => Err(format!("restore local symbol artifact: {error}")),
    }
}

async fn wait_for_fresh_local_symbol_artifact(
    search_plane: &SearchPlaneService,
    fingerprint: &str,
    active_fingerprint: Arc<RwLock<Option<String>>>,
    shutdown_requested: Arc<std::sync::atomic::AtomicBool>,
) -> Result<Option<UnifiedSymbolIndex>, String> {
    loop {
        if should_stop_build(&active_fingerprint, &shutdown_requested, fingerprint) {
            return Ok(None);
        }

        let search_status = search_plane
            .coordinator()
            .status_for(SearchCorpusKind::LocalSymbol);
        if search_status.fingerprint.as_deref() != Some(fingerprint) {
            return Ok(None);
        }

        match search_status.phase {
            SearchPlanePhase::Ready | SearchPlanePhase::Degraded => {
                return xiuxian_wendao::search::restore_local_symbol_hits(search_plane)
                    .await
                    .map(|hits| {
                        let hits = hits.into_iter().map(Into::into).collect::<Vec<_>>();
                        search::build_symbol_index_from_ast_hits(hits.as_slice())
                    })
                    .map(Some)
                    .map_err(|error| format!("restore local symbol artifact: {error}"));
            }
            SearchPlanePhase::Failed => {
                return Err(search_status.last_error.unwrap_or_else(|| {
                    format!(
                        "search-plane local symbol build failed for fingerprint `{fingerprint}`"
                    )
                }));
            }
            SearchPlanePhase::Idle | SearchPlanePhase::Indexing => {
                tokio::time::sleep(LOCAL_SYMBOL_RESTORE_POLL_INTERVAL).await;
            }
        }
    }
}

fn current_local_symbol_phase(
    search_plane: &SearchPlaneService,
    fingerprint: &str,
) -> SearchPlanePhase {
    let search_status = search_plane
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol);
    if search_status.fingerprint.as_deref() == Some(fingerprint) {
        return search_status.phase;
    }
    SearchPlanePhase::Ready
}

fn should_stop_build(
    active_fingerprint: &Arc<RwLock<Option<String>>>,
    shutdown_requested: &Arc<std::sync::atomic::AtomicBool>,
    fingerprint: &str,
) -> bool {
    let latest_fingerprint = active_fingerprint
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    shutdown_requested.load(Ordering::SeqCst) || latest_fingerprint.as_deref() != Some(fingerprint)
}

fn mark_symbol_index_failed(
    index_cache: &Arc<RwLock<Option<Arc<UnifiedSymbolIndex>>>>,
    status: &Arc<RwLock<SymbolIndexStatus>>,
    error: String,
) {
    *index_cache
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    *status
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = SymbolIndexStatus {
        phase: SymbolIndexPhase::Failed,
        last_error: Some(error),
        updated_at: Some(timestamp_now()),
    };
}
