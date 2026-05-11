pub(super) use crate::link_graph::QuantumAnchorHit;
pub(super) use crate::link_graph::addressing::SkeletonValidatedHit;
pub(super) use crate::zhenfa_router::native::agentic_nav::{
    generate_navigation_hint, render_agentic_nav_result, xml_escape,
};

pub(super) fn validated_hit(
    anchor_id: &str,
    vector_score: f64,
    is_valid: bool,
    anchor: &str,
    structural_path: Option<Vec<&str>>,
    reranked_score: f64,
) -> SkeletonValidatedHit {
    SkeletonValidatedHit {
        hit: QuantumAnchorHit {
            anchor_id: anchor_id.to_string(),
            vector_score,
        },
        is_valid,
        doc_id: "doc.md".to_string().into(),
        anchor: anchor.to_string(),
        structural_path: structural_path.map(|segments| {
            segments
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect()
        }),
        reranked_score,
    }
}
