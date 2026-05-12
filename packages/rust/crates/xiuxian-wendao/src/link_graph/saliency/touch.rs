//! `link_graph::saliency::touch` owns Wendao link graph saliency touch behavior.

use crate::link_graph::LinkGraphDisplayHit;
use crate::link_graph::runtime_config::resolve_link_graph_coactivation_runtime;
use crate::link_graph::saliency::{
    LinkGraphSaliencyPolicy, LinkGraphSaliencyTouchRequest, valkey_saliency_touch,
    valkey_saliency_touch_with_valkey,
};
use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, mpsc};
use std::thread;

/// Pre-resolved coactivation link emitted by search planning.
#[derive(Debug, Clone)]
pub struct SearchHitCoactivationLink {
    /// Source node id that was directly matched.
    pub source_node_id: String,
    /// Neighbor node id receiving a secondary activation.
    pub neighbor_node_id: String,
    /// Rank of the neighbor when it was pre-resolved.
    pub pre_resolved_rank: usize,
}

#[derive(Debug)]
enum SaliencyTouchTask {
    Runtime(LinkGraphSaliencyTouchRequest),
    WithValkey {
        request: LinkGraphSaliencyTouchRequest,
        valkey_url: String,
        key_prefix: Option<String>,
    },
}

fn touch_queue_sender() -> &'static mpsc::SyncSender<SaliencyTouchTask> {
    static SENDER: OnceLock<mpsc::SyncSender<SaliencyTouchTask>> = OnceLock::new();
    SENDER.get_or_init(|| {
        let runtime = resolve_link_graph_coactivation_runtime();
        let depth = runtime.touch_queue_depth.max(1);
        let (tx, rx) = mpsc::sync_channel::<SaliencyTouchTask>(depth);
        let builder = thread::Builder::new().name("link-graph-saliency-touch".to_string());
        if let Err(err) = builder.spawn(move || {
            for task in rx {
                execute_touch_task(task);
            }
        }) {
            log::error!("Failed to spawn saliency touch worker: {err}");
        }
        tx
    })
}

fn execute_touch_task(task: SaliencyTouchTask) {
    match task {
        SaliencyTouchTask::Runtime(request) => {
            if let Err(err) = valkey_saliency_touch(request) {
                log::error!("Failed to touch search hit during evolution: {err}");
            }
        }
        SaliencyTouchTask::WithValkey {
            request,
            valkey_url,
            key_prefix,
        } => {
            if let Err(err) =
                valkey_saliency_touch_with_valkey(request, &valkey_url, key_prefix.as_deref())
            {
                log::error!("Failed to touch search hit during evolution: {err}");
            }
        }
    }
}

fn enqueue_touch(task: SaliencyTouchTask) {
    let sender = touch_queue_sender();
    match sender.try_send(task) {
        Ok(()) => {}
        Err(mpsc::TrySendError::Full(task) | mpsc::TrySendError::Disconnected(task)) => {
            execute_touch_task(task);
        }
    }
}

fn enqueue_touch_request(
    request: LinkGraphSaliencyTouchRequest,
    valkey_url: Option<&str>,
    key_prefix: Option<&str>,
) {
    if let Some(valkey_url) = valkey_url {
        enqueue_touch(SaliencyTouchTask::WithValkey {
            request,
            valkey_url: valkey_url.to_string(),
            key_prefix: key_prefix.map(str::to_string),
        });
    } else {
        enqueue_touch(SaliencyTouchTask::Runtime(request));
    }
}

fn touch_hits_with_coactivation(
    hits: &[LinkGraphDisplayHit],
    links: &[SearchHitCoactivationLink],
    valkey_url: Option<&str>,
    key_prefix: Option<&str>,
) {
    if hits.is_empty() && links.is_empty() {
        return;
    }

    let mut direct_nodes: HashSet<String> = HashSet::new();
    for hit in hits {
        let node_id = hit.stem.trim();
        if node_id.is_empty() {
            continue;
        }
        direct_nodes.insert(node_id.to_string());
    }

    for node_id in &direct_nodes {
        enqueue_touch_request(
            LinkGraphSaliencyTouchRequest {
                node_id: node_id.clone(),
                activation_delta: 1,
                ..Default::default()
            },
            valkey_url,
            key_prefix,
        );
    }

    if links.is_empty() {
        return;
    }

    let runtime = resolve_link_graph_coactivation_runtime();
    if !runtime.enabled {
        return;
    }

    let base_alpha = LinkGraphSaliencyPolicy::default().alpha * runtime.alpha_scale;
    if base_alpha <= f64::EPSILON {
        return;
    }

    let mut neighbor_ranks: HashMap<String, usize> = HashMap::new();
    for link in links {
        let neighbor_id = link.neighbor_node_id.trim();
        if neighbor_id.is_empty() {
            continue;
        }
        if direct_nodes.contains(neighbor_id) {
            continue;
        }
        let source_id = link.source_node_id.trim();
        if !source_id.is_empty() && source_id == neighbor_id {
            continue;
        }

        neighbor_ranks
            .entry(neighbor_id.to_string())
            .and_modify(|rank| *rank = (*rank).min(link.pre_resolved_rank))
            .or_insert(link.pre_resolved_rank);
    }

    for (neighbor_id, rank) in neighbor_ranks {
        let rank_f64 = f64::from(u32::try_from(rank).unwrap_or(u32::MAX));
        let weight = 1.0 / (rank_f64 + 1.0);
        let alpha = base_alpha * weight;
        if alpha <= f64::EPSILON {
            continue;
        }

        enqueue_touch_request(
            LinkGraphSaliencyTouchRequest {
                node_id: neighbor_id,
                activation_delta: 1,
                alpha: Some(alpha),
                ..Default::default()
            },
            valkey_url,
            key_prefix,
        );
    }
}

/// Asynchronously touches a set of search hits to trigger saliency evolution.
///
/// This follows the Hebbian learning principle where frequently retrieved nodes
/// gain higher structural authority over time.
pub fn touch_search_hits_async(hits: &[LinkGraphDisplayHit], _links: &[SearchHitCoactivationLink]) {
    touch_hits_with_coactivation(hits, &[], None, None);
}

/// Asynchronously touches a set of search hits with explicit Valkey settings.
///
/// This follows the Hebbian learning principle where frequently retrieved nodes
/// gain higher structural authority over time.
pub fn touch_search_hits_async_with_valkey(
    hits: &[LinkGraphDisplayHit],
    links: &[SearchHitCoactivationLink],
    valkey_url: &str,
    key_prefix: Option<&str>,
) {
    if !links.is_empty() {
        match valkey_url.trim() {
            "" => return,
            trimmed => return touch_hits_with_coactivation(hits, links, Some(trimmed), key_prefix),
        }
    }
    let trimmed = valkey_url.trim();
    if trimmed.is_empty() {
        return;
    }
    touch_hits_with_coactivation(hits, &[], Some(trimmed), key_prefix);
}

/// Touches direct hits and pre-resolved neighbor activations asynchronously.
pub fn touch_search_hits_with_coactivation_async(
    hits: &[LinkGraphDisplayHit],
    links: &[SearchHitCoactivationLink],
) {
    touch_hits_with_coactivation(hits, links, None, None);
}

/// Touches direct hits and pre-resolved neighbor activations with explicit Valkey settings.
pub fn touch_search_hits_with_coactivation_async_with_valkey(
    hits: &[LinkGraphDisplayHit],
    links: &[SearchHitCoactivationLink],
    valkey_url: &str,
    key_prefix: Option<&str>,
) {
    let trimmed = valkey_url.trim();
    if trimmed.is_empty() {
        return;
    }
    touch_hits_with_coactivation(hits, links, Some(trimmed), key_prefix);
}
