//! Deterministic target-intent classification for Episteme reasoning slots.

pub(super) const OBJECT_SEED_KIND: &str = "object_proposal_slot";
pub(super) const RELATION_SEED_KIND: &str = "relation_proposal_slot";
pub(super) const SERVICE_CATALOG_SEED_KIND: &str = "service_catalog_review_slot";
pub(super) const OBJECT_INSTANCE_SEED_KIND: &str = "object_instance_review_slot";

pub(super) const OBJECT_FIELD_GROUP: &str = "object_proposal";
pub(super) const RELATION_FIELD_GROUP: &str = "relation_proposal";
pub(super) const SERVICE_CATALOG_FIELD_GROUP: &str = "service_catalog_review";
pub(super) const OBJECT_INSTANCE_FIELD_GROUP: &str = "object_instance_review";

pub(super) const TARGET_INTENT_COARSE_DOCUMENT_REVIEW: &str = "coarse_document_review";
pub(super) const TARGET_INTENT_OBJECT_TYPE_CANDIDATE: &str = "object_type_candidate";
pub(super) const TARGET_INTENT_RELATION_TYPE_CANDIDATE: &str = "relation_type_candidate";
pub(super) const TARGET_INTENT_SERVICE_CATALOG_EXTRACTION: &str = "service_catalog_extraction";
pub(super) const TARGET_INTENT_OBJECT_INSTANCE_CANDIDATE: &str = "object_instance_candidate";
pub(super) const TARGET_INTENT_POLICY_CITY_RELATION: &str = "policy_city_relation";
pub(super) const TARGET_INTENT_TABLE_ROW_EVIDENCE: &str = "table_row_evidence";

pub(super) const HINT_COARSE_DOCUMENT: &str = "document_root:coarse_document";
pub(super) const HINT_OBJECT_TYPE: &str = "document_root:object_type_candidate";
pub(super) const HINT_RELATION_TYPE: &str = "document_root:relation_type_candidate";
pub(super) const HINT_SERVICE_CATALOG: &str = "document_root:service_catalog";
pub(super) const HINT_OBJECT_INSTANCE: &str = "document_root:object_instance";
pub(super) const HINT_POLICY_CITY: &str = "document_root:policy_city_relation";
pub(super) const HINT_TABLE_ROW: &str = "document_root:table_row";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct ReasoningTarget {
    pub evidence_target_intent: &'static str,
    pub evidence_structure_hint: &'static str,
}

pub(super) fn classify_document_target(
    relative_path: &str,
    category: &str,
    extraction_route: &str,
) -> ReasoningTarget {
    let normalized = format!(
        "{} {} {}",
        relative_path.to_ascii_lowercase(),
        category.to_ascii_lowercase(),
        extraction_route.to_ascii_lowercase()
    );
    let original = format!("{relative_path} {category} {extraction_route}");
    let table_like = has_any(
        normalized.as_str(),
        &[".csv", ".tsv", ".xls", ".xlsx", "table", "catalog", "sheet"],
    ) || has_any(original.as_str(), &["表", "目录", "清单", "台账"]);
    let service_like = has_any(normalized.as_str(), &["service", "care"])
        || has_any(original.as_str(), &["服务", "护理", "照护", "康养", "养老"]);
    let policy_like = has_any(normalized.as_str(), &["policy", "rule"])
        || has_any(original.as_str(), &["政策", "办法", "规范"]);
    let city_like = has_any(normalized.as_str(), &["city", "pilot"])
        || has_any(original.as_str(), &["城市", "试点", "地区"]);
    let case_like =
        has_any(normalized.as_str(), &["case"]) || has_any(original.as_str(), &["案例"]);
    let object_type_like = has_any(
        normalized.as_str(),
        &["object_type", "schema", "ontology", "definition"],
    ) || has_any(original.as_str(), &["对象类型", "定义", "本体"]);
    let relation_type_like = has_any(normalized.as_str(), &["relation", "link_type"])
        || has_any(original.as_str(), &["关系", "关联"]);

    if policy_like && city_like {
        return ReasoningTarget {
            evidence_target_intent: TARGET_INTENT_POLICY_CITY_RELATION,
            evidence_structure_hint: HINT_POLICY_CITY,
        };
    }
    if service_like && table_like {
        return ReasoningTarget {
            evidence_target_intent: TARGET_INTENT_SERVICE_CATALOG_EXTRACTION,
            evidence_structure_hint: HINT_SERVICE_CATALOG,
        };
    }
    if case_like {
        return ReasoningTarget {
            evidence_target_intent: TARGET_INTENT_OBJECT_INSTANCE_CANDIDATE,
            evidence_structure_hint: HINT_OBJECT_INSTANCE,
        };
    }
    if object_type_like {
        return ReasoningTarget {
            evidence_target_intent: TARGET_INTENT_OBJECT_TYPE_CANDIDATE,
            evidence_structure_hint: HINT_OBJECT_TYPE,
        };
    }
    if relation_type_like {
        return ReasoningTarget {
            evidence_target_intent: TARGET_INTENT_RELATION_TYPE_CANDIDATE,
            evidence_structure_hint: HINT_RELATION_TYPE,
        };
    }
    if table_like {
        return ReasoningTarget {
            evidence_target_intent: TARGET_INTENT_TABLE_ROW_EVIDENCE,
            evidence_structure_hint: HINT_TABLE_ROW,
        };
    }

    ReasoningTarget {
        evidence_target_intent: TARGET_INTENT_COARSE_DOCUMENT_REVIEW,
        evidence_structure_hint: HINT_COARSE_DOCUMENT,
    }
}

pub(super) fn seed_kinds_for_target_intent(intent: &str) -> &'static [&'static str] {
    match intent {
        TARGET_INTENT_OBJECT_TYPE_CANDIDATE => &[OBJECT_SEED_KIND],
        TARGET_INTENT_RELATION_TYPE_CANDIDATE | TARGET_INTENT_POLICY_CITY_RELATION => {
            &[RELATION_SEED_KIND]
        }
        TARGET_INTENT_SERVICE_CATALOG_EXTRACTION => &[SERVICE_CATALOG_SEED_KIND],
        TARGET_INTENT_OBJECT_INSTANCE_CANDIDATE | TARGET_INTENT_TABLE_ROW_EVIDENCE => {
            &[OBJECT_INSTANCE_SEED_KIND]
        }
        _ => &[OBJECT_SEED_KIND, RELATION_SEED_KIND],
    }
}

pub(super) fn target_field_group(seed_kind: &str) -> Option<&'static str> {
    match seed_kind {
        OBJECT_SEED_KIND => Some(OBJECT_FIELD_GROUP),
        RELATION_SEED_KIND => Some(RELATION_FIELD_GROUP),
        SERVICE_CATALOG_SEED_KIND => Some(SERVICE_CATALOG_FIELD_GROUP),
        OBJECT_INSTANCE_SEED_KIND => Some(OBJECT_INSTANCE_FIELD_GROUP),
        _ => None,
    }
}

pub(super) fn default_evidence_target_intent() -> String {
    TARGET_INTENT_COARSE_DOCUMENT_REVIEW.to_owned()
}

pub(super) fn default_evidence_anchor_kind() -> String {
    "document_root".to_owned()
}

pub(super) fn default_evidence_structure_hint() -> String {
    HINT_COARSE_DOCUMENT.to_owned()
}

fn has_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
