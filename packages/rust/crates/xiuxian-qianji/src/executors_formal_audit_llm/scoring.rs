use xiuxian_zhenfa::ZhenfaTransmuter;

pub(super) const FORMAL_AUDIT_XML_SCORE_CONTRACT: &str = "Return XML only. Include exactly one numeric <score> tag with a value in [0.0, 1.0]. Include a <reason> tag that explains the score. Do not emit Markdown or prose outside XML.";

pub(super) fn extract_xml_score(text: &str) -> Option<f32> {
    ZhenfaTransmuter::get_tag_f32(text, "score")
}

pub(super) fn score_to_memrl_reward(score: f32) -> f32 {
    score.clamp(0.0, 1.0)
}
