use crate::analyzers::DocRecord;
use crate::analyzers::ProjectionPageKind;

use super::anchors::TargetAnchors;
use crate::analyzers::projection::contracts::projection_kind_from_doc_format;

pub(super) fn projection_kind_token(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "reference",
        ProjectionPageKind::HowTo => "howto",
        ProjectionPageKind::Tutorial => "tutorial",
        ProjectionPageKind::Explanation => "explanation",
    }
}

pub(super) fn doc_projection_kind(doc: &DocRecord, targets: &TargetAnchors) -> ProjectionPageKind {
    let kind = projection_kind_from_doc_format(doc.format.as_deref());
    if kind == ProjectionPageKind::Explanation && !targets.symbol_ids.is_empty() {
        ProjectionPageKind::Reference
    } else {
        kind
    }
}
