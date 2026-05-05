use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum SaliencyCommand {
    /// Read a saliency state by node id.
    Get { node_id: String },
    /// Settle all persisted saliency states forward in time.
    Decay {
        #[arg(long)]
        now_unix: Option<i64>,
    },
    /// Touch a node and update saliency with decay + activation.
    Touch {
        node_id: String,
        #[arg(long, default_value_t = 1)]
        activation_delta: u64,
        #[arg(long)]
        saliency_base: Option<f64>,
        #[arg(long)]
        decay_rate: Option<f64>,
        #[arg(long)]
        alpha: Option<f64>,
        #[arg(long)]
        minimum_saliency: Option<f64>,
        #[arg(long)]
        maximum_saliency: Option<f64>,
        #[arg(long)]
        now_unix: Option<i64>,
    },
}
