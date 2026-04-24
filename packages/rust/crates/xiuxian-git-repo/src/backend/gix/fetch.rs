use gix::remote::Direction;

use super::constants::ORIGIN_REMOTE_NAME;
use super::error::{BackendError, error_message};
use super::interrupt::run_interruptible_remote_operation;
use super::retry::retry_remote_operation;
use super::types::RepositoryHandle;

pub(crate) fn fetch_origin_with_retry(repository: &RepositoryHandle) -> Result<(), BackendError> {
    retry_remote_operation(|| fetch_origin_once(repository))
}

fn fetch_origin_once(repository: &RepositoryHandle) -> Result<(), BackendError> {
    run_interruptible_remote_operation("fetch origin", |should_interrupt| {
        repository
            .find_remote(ORIGIN_REMOTE_NAME)
            .map_err(error_message)?
            .with_fetch_tags(gix::remote::fetch::Tags::All)
            .connect(Direction::Fetch)
            .map_err(error_message)?
            .prepare_fetch(
                gix::progress::Discard,
                gix::remote::ref_map::Options::default(),
            )
            .map_err(error_message)?
            .receive(gix::progress::Discard, should_interrupt)
            .map_err(error_message)?;
        Ok(())
    })
}
