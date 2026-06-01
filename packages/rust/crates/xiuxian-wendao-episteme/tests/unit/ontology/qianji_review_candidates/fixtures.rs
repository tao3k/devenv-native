pub(super) fn qianji_zero_candidate_review_artifact() -> &'static str {
    r#"{
  "schema": "qianji.openai_compatible_llm_response.v1",
  "model": "deepseek/deepseek-v4-pro",
  "activity_id": "activity.episteme_ontology_reasoning_fill.test",
  "content": "{}",
  "episteme_review": {
    "schema": "xiuxian.wendao.episteme.reasoning_fill_review.v1",
    "status": "review_only",
    "fillItemId": "structural_facts.reasoning_fill_plan.test",
    "targetLedgerFieldGroup": "object_proposal",
    "reviewSummary": "Evidence is not enough for a safe ObjectType proposal.",
    "candidatePatchCount": 0,
    "candidatePatches": [],
    "blockers": [
      "Evidence describes procedural service items rather than object type boundaries",
      "No stable property set is explicit in the source evidence"
    ],
    "rdfMutation": false
  }
}"#
}

pub(super) fn qianji_review_artifact() -> &'static str {
    r#"{
  "schema": "qianji.openai_compatible_llm_response.v1",
  "model": "deepseek/deepseek-v4-pro",
  "activity_id": "activity.episteme_ontology_reasoning_fill.test",
  "content": "{}",
  "episteme_review": {
    "schema": "xiuxian.wendao.episteme.reasoning_fill_review.v1",
    "status": "review_only",
    "fillItemId": "structural_facts.reasoning_fill_plan.test",
    "targetLedgerFieldGroup": "object_proposal",
    "reviewSummary": "Evidence supports one object candidate.",
    "candidatePatchCount": 1,
    "candidatePatches": [
      {
        "patchKind": "object_model_object_type_candidate",
        "fillItemId": "structural_facts.reasoning_fill_plan.test",
        "targetLedgerFieldGroup": "object_proposal",
        "objectType": {
          "domain": "episteme://medical-extension/ltc",
          "apiName": "LtcServiceCatalog",
          "displayName": "Shanghai LTC Service Catalog",
          "pluralDisplayName": "Shanghai LTC Service Catalogs",
          "status": "preview",
          "rdfClass": "https://wendao.ai/private/medical/ltc#ServiceCatalog",
          "primaryKey": ["sourceId"],
          "displayNameProperty": "name",
          "titleProperty": "name",
          "interfaces": [],
          "visibility": "private"
        },
        "sourceEvidence": [
          {
            "fileId": "ltc.file.policy.001",
            "relativePath": "policy/source.doc",
            "quote": "raw private quote body",
            "reason": "supports this candidate"
          }
        ],
        "confidence": "high",
        "reviewNotes": "Review-only candidate."
      }
    ],
    "blockers": [],
    "rdfMutation": false
  },
  "provider_response": {}
}"#
}
