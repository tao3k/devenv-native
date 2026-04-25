use gix::remote::Direction;

use super::constants::{MIRROR_FETCH_REFSPEC, ORIGIN_REMOTE_NAME};
use super::error::{BackendError, error_message};
use super::interrupt::run_interruptible_remote_operation;
use super::retry::retry_remote_operation;
use super::types::RepositoryHandle;

pub(crate) fn fetch_origin_with_retry(repository: &RepositoryHandle) -> Result<(), BackendError> {
    retry_remote_operation(|| fetch_origin_once(repository))
}

fn fetch_origin_once(repository: &RepositoryHandle) -> Result<(), BackendError> {
    run_interruptible_remote_operation("fetch origin", |should_interrupt| {
        let mut ref_map_options = gix::remote::ref_map::Options::default();
        let fetch_refspec = managed_fetch_refspec(repository);
        ref_map_options.extra_refspecs.push(
            gix::refspec::parse(
                fetch_refspec.as_str().into(),
                gix::refspec::parse::Operation::Fetch,
            )
            .map_err(error_message)?
            .into(),
        );
        repository
            .find_remote(ORIGIN_REMOTE_NAME)
            .map_err(error_message)?
            .with_fetch_tags(gix::remote::fetch::Tags::All)
            .connect(Direction::Fetch)
            .map_err(error_message)?
            .prepare_fetch(gix::progress::Discard, ref_map_options)
            .map_err(error_message)?
            .receive(gix::progress::Discard, should_interrupt)
            .map_err(error_message)?;
        Ok(())
    })
}

fn managed_fetch_refspec(repository: &RepositoryHandle) -> String {
    if repository.is_bare() {
        MIRROR_FETCH_REFSPEC.to_string()
    } else {
        format!("+refs/heads/*:refs/remotes/{ORIGIN_REMOTE_NAME}/*")
    }
}
