use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::sync::Notify;
use xiuxian_wendao_attachments::pdf::ocr::PdfOcrShardResult;

#[derive(Debug, Default)]
pub(super) struct InFlightShardRegistry {
    entries: Mutex<HashMap<String, Arc<InFlightShardEntry>>>,
}

#[derive(Debug)]
pub(super) enum InFlightShardReservation {
    Owner {
        key: String,
        entry: Arc<InFlightShardEntry>,
    },
    Waiter {
        entry: Arc<InFlightShardEntry>,
    },
}

#[derive(Debug)]
pub(super) struct InFlightShardEntry {
    state: Mutex<Option<Result<PdfOcrShardResult, String>>>,
    notify: Notify,
}

impl InFlightShardRegistry {
    pub(super) fn reserve(&self, key: String) -> InFlightShardReservation {
        let mut entries = self.lock_entries();
        if let Some(entry) = entries.get(&key) {
            return InFlightShardReservation::Waiter {
                entry: Arc::clone(entry),
            };
        }
        let entry = Arc::new(InFlightShardEntry {
            state: Mutex::new(None),
            notify: Notify::new(),
        });
        entries.insert(key.clone(), Arc::clone(&entry));
        InFlightShardReservation::Owner { key, entry }
    }

    pub(super) fn publish(
        &self,
        key: &str,
        entry: &Arc<InFlightShardEntry>,
        result: Result<PdfOcrShardResult, String>,
    ) {
        {
            let mut state = entry.lock_state();
            *state = Some(result);
        }
        entry.notify.notify_waiters();

        let mut entries = self.lock_entries();
        if entries
            .get(key)
            .is_some_and(|current| Arc::ptr_eq(current, entry))
        {
            entries.remove(key);
        }
    }

    pub(super) fn len(&self) -> usize {
        self.lock_entries().len()
    }

    fn lock_entries(&self) -> MutexGuard<'_, HashMap<String, Arc<InFlightShardEntry>>> {
        match self.entries.lock() {
            Ok(entries) => entries,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

impl InFlightShardEntry {
    pub(super) async fn wait(&self) -> Result<PdfOcrShardResult, String> {
        loop {
            let completed_result = { self.lock_state().clone() };
            if let Some(result) = completed_result {
                return result;
            }
            self.notify.notified().await;
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, Option<Result<PdfOcrShardResult, String>>> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/inflight.rs"]
mod tests;
