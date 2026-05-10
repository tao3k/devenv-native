use super::{
    AstSearchHit, AttachmentSearchHit, DefinitionSearchHit, IntentSearchHit, KnowledgeSearchHit,
    ObservationHint, ReferenceSearchHit, SearchBacklinkItem, SearchHit,
};
use xiuxian_wendao::search::contracts as domain;

impl From<domain::AttachmentSearchHit> for AttachmentSearchHit {
    fn from(value: domain::AttachmentSearchHit) -> Self {
        Self {
            name: value.name,
            path: value.path.into(),
            source_id: value.source_id.into(),
            source_stem: value.source_stem,
            source_title: value.source_title,
            source_path: value.source_path.into(),
            attachment_id: value.attachment_id.into(),
            attachment_path: value.attachment_path.into(),
            attachment_name: value.attachment_name,
            attachment_ext: value.attachment_ext,
            kind: value.kind.into(),
            navigation_target: value.navigation_target.into(),
            score: value.score,
            vision_snippet: value.vision_snippet,
        }
    }
}

impl From<domain::AstSearchHit> for AstSearchHit {
    fn from(value: domain::AstSearchHit) -> Self {
        Self {
            name: value.name,
            signature: value.signature,
            path: value.path,
            language: value.language,
            crate_name: value.crate_name,
            project_name: value.project_name,
            root_label: value.root_label,
            node_kind: value.node_kind.map(Into::into),
            owner_title: value.owner_title,
            navigation_target: value.navigation_target.into(),
            line_start: value.line_start,
            line_end: value.line_end,
            score: value.score,
        }
    }
}

impl From<AstSearchHit> for domain::AstSearchHit {
    fn from(value: AstSearchHit) -> Self {
        Self {
            name: value.name,
            signature: value.signature,
            path: value.path,
            language: value.language,
            crate_name: value.crate_name,
            project_name: value.project_name,
            root_label: value.root_label,
            node_kind: value.node_kind.map(|kind| kind.into_string()),
            owner_title: value.owner_title,
            navigation_target: value.navigation_target.into(),
            line_start: value.line_start,
            line_end: value.line_end,
            score: value.score,
        }
    }
}

#[cfg(all(test, feature = "zhenfa-router"))]
pub(crate) fn domain_ast_hits_for_search_plane(
    hits: Vec<AstSearchHit>,
) -> Vec<domain::AstSearchHit> {
    hits.into_iter().map(Into::into).collect()
}

impl From<domain::DefinitionSearchHit> for DefinitionSearchHit {
    fn from(value: domain::DefinitionSearchHit) -> Self {
        Self {
            name: value.name,
            signature: value.signature,
            path: value.path,
            language: value.language,
            crate_name: value.crate_name,
            project_name: value.project_name,
            root_label: value.root_label,
            node_kind: value.node_kind.map(Into::into),
            owner_title: value.owner_title,
            navigation_target: value.navigation_target.into(),
            line_start: value.line_start,
            line_end: value.line_end,
            score: value.score,
            observation_hints: value
                .observation_hints
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<domain::ObservationHint> for ObservationHint {
    fn from(value: domain::ObservationHint) -> Self {
        Self {
            language: value.language,
            scope: value.scope,
            pattern: value.pattern,
        }
    }
}

impl From<domain::ReferenceSearchHit> for ReferenceSearchHit {
    fn from(value: domain::ReferenceSearchHit) -> Self {
        Self {
            name: value.name,
            path: value.path,
            language: value.language,
            crate_name: value.crate_name,
            project_name: value.project_name,
            root_label: value.root_label,
            navigation_target: value.navigation_target.into(),
            line: value.line,
            column: value.column,
            line_text: value.line_text,
            score: value.score,
        }
    }
}

impl From<domain::KnowledgeSearchHit> for KnowledgeSearchHit {
    fn from(value: domain::KnowledgeSearchHit) -> Self {
        Self {
            id: value.id.into(),
            label: value.label,
            path: value.path,
            navigation_target: value.navigation_target.into(),
            score: value.score,
            snippet: value.snippet,
        }
    }
}

impl From<domain::SearchBacklinkItem> for SearchBacklinkItem {
    fn from(value: domain::SearchBacklinkItem) -> Self {
        Self {
            id: value.id,
            title: value.title,
            path: value.path.map(Into::into),
            kind: value.kind.map(Into::into),
        }
    }
}

impl From<domain::SearchHit> for SearchHit {
    fn from(value: domain::SearchHit) -> Self {
        Self {
            stem: value.stem,
            title: value.title,
            path: value.path.into(),
            doc_type: value.doc_type.map(Into::into),
            tags: value.tags.into_iter().map(Into::into).collect(),
            score: value.score,
            best_section: value.best_section,
            match_reason: value.match_reason,
            hierarchical_uri: value.hierarchical_uri,
            hierarchy: value.hierarchy,
            saliency_score: value.saliency_score,
            audit_status: value.audit_status.map(Into::into),
            verification_state: value.verification_state.map(Into::into),
            implicit_backlinks: value.implicit_backlinks,
            implicit_backlink_items: value
                .implicit_backlink_items
                .map(|items| items.into_iter().map(Into::into).collect()),
            navigation_target: value.navigation_target.map(Into::into),
        }
    }
}

impl From<domain::IntentSearchHit> for IntentSearchHit {
    fn from(value: domain::IntentSearchHit) -> Self {
        Self {
            label: value.label,
            action: value.action,
            score: value.score,
        }
    }
}
