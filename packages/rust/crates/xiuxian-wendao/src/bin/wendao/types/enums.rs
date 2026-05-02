use crate::{
    LinkGraphAttachmentKind, LinkGraphPprSubgraphMode, LinkGraphScope, LinkGraphSuggestedLinkState,
    analyzers::ProjectionPageKind,
};
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum OutputFormat {
    Text,
    Json,
    Pretty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum RelatedPprSubgraphModeArg {
    Auto,
    Disabled,
    Force,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum LinkGraphScopeArg {
    Mixed,
    DocOnly,
    SectionOnly,
}

impl From<LinkGraphScopeArg> for LinkGraphScope {
    fn from(value: LinkGraphScopeArg) -> Self {
        match value {
            LinkGraphScopeArg::Mixed => Self::Mixed,
            LinkGraphScopeArg::DocOnly => Self::DocOnly,
            LinkGraphScopeArg::SectionOnly => Self::SectionOnly,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum SuggestedLinkStateArg {
    Provisional,
    Promoted,
    Rejected,
}

impl From<SuggestedLinkStateArg> for LinkGraphSuggestedLinkState {
    fn from(value: SuggestedLinkStateArg) -> Self {
        match value {
            SuggestedLinkStateArg::Provisional => Self::Provisional,
            SuggestedLinkStateArg::Promoted => Self::Promoted,
            SuggestedLinkStateArg::Rejected => Self::Rejected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum DecisionTargetStateArg {
    Promoted,
    Rejected,
}

impl From<DecisionTargetStateArg> for LinkGraphSuggestedLinkState {
    fn from(value: DecisionTargetStateArg) -> Self {
        match value {
            DecisionTargetStateArg::Promoted => Self::Promoted,
            DecisionTargetStateArg::Rejected => Self::Rejected,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum AttachmentKindArg {
    Image,
    Pdf,
    Gpg,
    Document,
    Archive,
    Audio,
    Video,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ProjectionPageKindArg {
    Reference,
    HowTo,
    Tutorial,
    Explanation,
}

impl From<ProjectionPageKindArg> for ProjectionPageKind {
    fn from(value: ProjectionPageKindArg) -> Self {
        match value {
            ProjectionPageKindArg::Reference => Self::Reference,
            ProjectionPageKindArg::HowTo => Self::HowTo,
            ProjectionPageKindArg::Tutorial => Self::Tutorial,
            ProjectionPageKindArg::Explanation => Self::Explanation,
        }
    }
}

impl From<AttachmentKindArg> for LinkGraphAttachmentKind {
    fn from(value: AttachmentKindArg) -> Self {
        match value {
            AttachmentKindArg::Image => Self::Image,
            AttachmentKindArg::Pdf => Self::Pdf,
            AttachmentKindArg::Gpg => Self::Gpg,
            AttachmentKindArg::Document => Self::Document,
            AttachmentKindArg::Archive => Self::Archive,
            AttachmentKindArg::Audio => Self::Audio,
            AttachmentKindArg::Video => Self::Video,
            AttachmentKindArg::Other => Self::Other,
        }
    }
}

impl From<RelatedPprSubgraphModeArg> for LinkGraphPprSubgraphMode {
    fn from(value: RelatedPprSubgraphModeArg) -> Self {
        match value {
            RelatedPprSubgraphModeArg::Auto => Self::Auto,
            RelatedPprSubgraphModeArg::Disabled => Self::Disabled,
            RelatedPprSubgraphModeArg::Force => Self::Force,
        }
    }
}
