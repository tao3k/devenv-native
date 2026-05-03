use crate::studio::analysis::markdown::compile_markdown_ir;
use xiuxian_wendao::search::contracts::AnalysisNodeKind;

#[test]
fn compile_markdown_ir_emits_sections_code_blocks_and_atoms() {
    let compiled = compile_markdown_ir(
        "docs/sample.md",
        "# Heading\n\n```rust\nfn main() {}\n```\n",
    );

    assert_eq!(compiled.document_hash.len(), 64);
    assert!(
        compiled
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AnalysisNodeKind::Section))
    );
    assert!(
        compiled
            .retrieval_atoms
            .iter()
            .any(|atom| atom.semantic_type == "h1")
    );
    assert!(
        compiled
            .retrieval_atoms
            .iter()
            .any(|atom| atom.semantic_type == "code:rust")
    );
    assert!(compiled.diagnostics.is_empty());
}

#[test]
fn compile_markdown_ir_emits_display_math_atoms() {
    let compiled = compile_markdown_ir(
        "docs/math.md",
        "# Formula\n\n$$\nQ = clamp(round(R / S + Z), qmin, qmax)\n$$\n",
    );

    assert!(
        compiled
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AnalysisNodeKind::Math))
    );
    assert!(compiled.retrieval_atoms.iter().any(|atom| {
        atom.owner_id.starts_with("math:")
            && atom.semantic_type == "math:block"
            && atom.line_start == Some(3)
            && atom.line_end == Some(5)
    }));
}

#[test]
fn compile_markdown_ir_emits_observation_atoms() {
    let compiled = compile_markdown_ir(
        "docs/observations.md",
        "# Findings\n\n> Calibration drift exceeded the expected INT8 threshold.\n",
    );

    assert!(
        compiled
            .nodes
            .iter()
            .any(|node| matches!(node.kind, AnalysisNodeKind::Observation))
    );
    assert!(compiled.retrieval_atoms.iter().any(|atom| {
        atom.owner_id.starts_with("obs:")
            && atom.semantic_type == "observation"
            && atom.surface
                == Some(xiuxian_wendao::search::contracts::RetrievalChunkSurface::Observation)
    }));
}
