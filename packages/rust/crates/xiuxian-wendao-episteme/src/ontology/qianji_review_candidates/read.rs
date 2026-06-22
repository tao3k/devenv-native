//! Qianji review artifact loading.

use std::{fs, path::Path};

use anyhow::{Context, Result, bail};

use super::{
    types::{EPISTEME_REVIEW_SCHEMA, EpistemeReview, QIANJI_RESPONSE_SCHEMA, QianjiReviewArtifact},
    validate::validate_review,
};

pub(super) fn read_review_artifact(path: &Path) -> Result<EpistemeReview> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let artifact: QianjiReviewArtifact = serde_json::from_str(raw.as_str())
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if artifact.schema != QIANJI_RESPONSE_SCHEMA {
        bail!(
            "Qianji review artifact `{}` has unsupported schema `{}`",
            path.display(),
            artifact.schema
        );
    }
    let review = artifact.episteme_review.with_context(|| {
        format!(
            "Qianji review artifact `{}` has no episteme_review",
            path.display()
        )
    })?;
    if review.schema != EPISTEME_REVIEW_SCHEMA {
        bail!(
            "Qianji review artifact `{}` has unsupported episteme_review schema `{}`",
            path.display(),
            review.schema
        );
    }
    validate_review(&review, path)?;
    Ok(review)
}
