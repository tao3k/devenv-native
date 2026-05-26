use xiuxian_db_store::artifact_cache::ContentAddressedFilesystemBlobCache;
use xiuxian_qianhuan::{
    InjectionMode, InjectionPolicy, InjectionSessionId, InjectionSnapshot, InjectionSnapshotId,
    InjectionSnapshotInput, InjectionTurnId, PromptContextBlock, PromptContextBlockId,
    PromptContextBlockInput, PromptContextCategory, PromptContextPackIdentity, PromptContextSource,
    PromptSessionScope, fetch_through_injection_snapshot_pack,
    read_through_injection_snapshot_pack,
};

#[test]
fn prompt_context_pack_identity_is_stable_across_turn_ids() -> Result<(), Box<dyn std::error::Error>>
{
    let first = snapshot("snap-1", 1);
    let second = snapshot("snap-2", 2);

    let first_identity = PromptContextPackIdentity::from_snapshot_content(&first)?;
    let second_identity = PromptContextPackIdentity::from_snapshot_content(&second)?;

    assert_eq!(first_identity, second_identity);
    Ok(())
}

#[test]
fn prompt_context_pack_readthrough_hits_for_repeated_context()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path());
    let first_snapshot = snapshot("snap-1", 1);
    let repeated_snapshot = snapshot("snap-2", 2);

    let first = read_through_injection_snapshot_pack(&cache, &first_snapshot)?;
    assert!(!first.cache_hit());
    assert_eq!(first.key().namespace().as_str(), "agent");
    assert_eq!(
        first.key().kind().as_storage_component(),
        "prompt-context-pack"
    );
    let first_bytes = first.bytes().to_vec();
    let first_json: serde_json::Value = serde_json::from_slice(&first_bytes)?;
    assert_eq!(
        first_json["schema"],
        "xiuxian_qianhuan.prompt_context_pack.v1"
    );
    assert_eq!(first_json["session_id"], "session-a");
    assert!(first_json.get("snapshot_id").is_none());
    assert!(first_json.get("turn_id").is_none());

    let second = read_through_injection_snapshot_pack(&cache, &repeated_snapshot)?;
    assert!(second.cache_hit());
    assert_eq!(second.bytes(), first_bytes.as_slice());

    Ok(())
}

#[test]
fn prompt_context_pack_fetchthrough_hits_for_repeated_context()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path());
    let first_snapshot = snapshot("snap-1", 1);
    let repeated_snapshot = snapshot("snap-2", 2);

    let first = fetch_through_injection_snapshot_pack(&cache, first_snapshot)?;
    assert!(!first.cache_hit());
    assert_eq!(first.artifact().backend_name(), "filesystem");
    assert_eq!(
        first
            .artifact()
            .write_outcome()
            .map(|write| write.byte_len()),
        Some(first.byte_len())
    );

    let first_bytes = first.bytes().to_vec();
    let second = fetch_through_injection_snapshot_pack(&cache, repeated_snapshot)?;
    assert!(second.cache_hit());
    assert_eq!(second.artifact().backend_name(), "filesystem");
    assert_eq!(second.bytes(), first_bytes.as_slice());

    Ok(())
}

fn snapshot(snapshot_id: &str, turn_id: u64) -> InjectionSnapshot {
    InjectionSnapshot::from_blocks(InjectionSnapshotInput {
        snapshot_id: InjectionSnapshotId::new(snapshot_id),
        session_id: InjectionSessionId::new("session-a"),
        turn_id: InjectionTurnId::new(turn_id),
        policy: InjectionPolicy {
            mode: InjectionMode::Classified,
            ..InjectionPolicy::default()
        },
        role_mix: None,
        blocks: vec![
            block(
                "policy",
                PromptContextSource::Policy,
                PromptContextCategory::Policy,
                950,
                "policy context",
                true,
            ),
            block(
                "knowledge",
                PromptContextSource::Knowledge,
                PromptContextCategory::Knowledge,
                840,
                "retrieved knowledge context",
                false,
            ),
        ],
    })
}

fn block(
    block_id: &str,
    source: PromptContextSource,
    category: PromptContextCategory,
    priority: u16,
    payload: &str,
    anchor: bool,
) -> PromptContextBlock {
    PromptContextBlock::new(PromptContextBlockInput {
        block_id: PromptContextBlockId::new(block_id),
        source,
        category,
        priority,
        session_scope: PromptSessionScope::new("session-a"),
        payload: payload.to_owned(),
        anchor,
    })
}
