use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::EpistemeFileRow;

use super::model::CacheEvidence;

pub(super) fn validate_run_id(value: &str) -> anyhow::Result<()> {
    let safe = !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if safe {
        Ok(())
    } else {
        anyhow::bail!("invalid ontology candidate generation run id `{value}`")
    }
}

pub(super) fn mapping_term_candidate_id(stable_key: &str) -> String {
    format!("ontology.term.{}", short_hash(stable_key))
}

pub(super) fn source_candidate_id(file_id: &str) -> String {
    format!("ontology.source.{}", short_hash(file_id))
}

pub(super) fn evidence_candidate_id(run_id: &str, queue_id: &str) -> String {
    format!(
        "ontology.evidence.{}",
        short_hash(format!("{run_id}:{queue_id}").as_str())
    )
}

pub(super) fn relation_candidate_id(
    kind: &str,
    source: &str,
    target: &str,
    evidence: &str,
) -> String {
    format!(
        "ontology.relation.{}",
        short_hash(format!("{kind}:{source}:{target}:{evidence}").as_str())
    )
}

pub(super) fn source_revision(
    source_contract_id: &str,
    files: &[EpistemeFileRow],
    mapping_ledger: &str,
    evidence: &[CacheEvidence],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_contract_id.as_bytes());
    for file in files {
        hasher.update(file.file_id.as_bytes());
        hasher.update(file.relative_path.as_bytes());
        hasher.update(file.sha256.as_bytes());
    }
    hasher.update(mapping_ledger.as_bytes());
    for row in evidence {
        hasher.update(row.run_id.as_bytes());
        hasher.update(row.queue_id.as_bytes());
        hasher.update(row.text_sha256.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

pub(super) fn sha256_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(super) fn short_hash(value: &str) -> String {
    sha256_text(value).chars().take(16).collect()
}

pub(super) fn org_uuid(value: &str) -> String {
    let hash = sha256_text(value);
    format!(
        "{}-{}-{}-{}-{}",
        &hash[0..8],
        &hash[8..12],
        &hash[12..16],
        &hash[16..20],
        &hash[20..32]
    )
}

pub(super) fn tsv(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ").trim().to_string()
}

pub(super) fn org_cell(value: &str) -> String {
    value.replace('|', "\\vert{}")
}
