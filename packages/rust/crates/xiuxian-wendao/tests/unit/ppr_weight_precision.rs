//! Precision regression for weighted-seed PPR ranking.
use serial_test::serial;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao::link_graph::{
    LinkGraphIndex, LinkGraphNeighbor, LinkGraphRelatedPprOptions, LinkGraphSaliencyTouchRequest,
    set_link_graph_wendao_config_override, valkey_saliency_touch_with_valkey,
};

const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/0";

#[test]
#[serial(link_graph_runtime_config)]
fn test_ppr_weight_precision_impact() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let outcome = (|| -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();

        for (name, content) in [
            ("alpha-seed", "Alpha seed linking to [[alpha-target]]."),
            ("alpha-target", "Alpha target node."),
            ("zeta-seed", "Zeta seed linking to [[zeta-target]]."),
            ("zeta-target", "Zeta target node."),
        ] {
            fs::write(root.join(format!("{name}.md")), content)?;
        }

        let config_path = root.join("wendao-test.toml");
        fs::write(
            &config_path,
            format!(
                "[link_graph.cache]\nvalkey_url = \"{TEST_VALKEY_URL}\"\nkey_prefix = \"{prefix}\"\n"
            ),
        )?;
        let config_path_string = config_path.to_string_lossy().to_string();
        set_link_graph_wendao_config_override(&config_path_string);

        let index = LinkGraphIndex::build(root)?;
        let seeds = vec!["alpha-seed".to_string(), "zeta-seed".to_string()];
        let ppr_options = LinkGraphRelatedPprOptions {
            alpha: Some(0.15),
            ..Default::default()
        };

        let (baseline_rows, _) =
            index.related_from_seeds_with_diagnostics(&seeds, 2, 10, Some(&ppr_options));
        let baseline_stems = top_stems(&baseline_rows);
        assert_eq!(
            baseline_stems.first().map(String::as_str),
            Some("alpha-target")
        );
        assert_eq!(
            baseline_stems.get(1).map(String::as_str),
            Some("zeta-target")
        );

        valkey_saliency_touch_with_valkey(
            LinkGraphSaliencyTouchRequest {
                node_id: "zeta-seed".to_string(),
                activation_delta: 64,
                saliency_base: Some(128.0),
                alpha: Some(0.75),
                minimum_saliency: Some(1.0),
                maximum_saliency: Some(256.0),
                now_unix: Some(1_700_000_000),
                ..Default::default()
            },
            TEST_VALKEY_URL,
            Some(&prefix),
        )
        .map_err(std::io::Error::other)?;

        let (weighted_rows, _) =
            index.related_from_seeds_with_diagnostics(&seeds, 2, 10, Some(&ppr_options));
        let weighted_stems = top_stems(&weighted_rows);
        assert_eq!(
            weighted_stems.first().map(String::as_str),
            Some("zeta-target")
        );
        assert!(
            position_of(&weighted_stems, "zeta-target")
                < position_of(&weighted_stems, "alpha-target")
        );

        Ok(())
    })();

    let _ = clear_prefix(&prefix);
    outcome
}

fn top_stems(rows: &[LinkGraphNeighbor]) -> Vec<String> {
    rows.iter().take(2).map(|row| row.stem.clone()).collect()
}

fn position_of(stems: &[String], needle: &str) -> usize {
    stems
        .iter()
        .position(|stem| stem == needle)
        .unwrap_or(usize::MAX)
}

fn unique_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_nanos());
    format!("omni:test:ppr-weight-precision:{nanos}")
}

fn clear_prefix(prefix: &str) -> Result<(), String> {
    let client = redis::Client::open(TEST_VALKEY_URL).map_err(|err| err.to_string())?;
    let mut conn = client.get_connection().map_err(|err| err.to_string())?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern)
        .query(&mut conn)
        .map_err(|err| err.to_string())?;
    if !keys.is_empty() {
        redis::cmd("DEL")
            .arg(keys)
            .query::<()>(&mut conn)
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}
