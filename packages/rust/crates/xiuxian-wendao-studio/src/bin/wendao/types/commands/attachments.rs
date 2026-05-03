use crate::bin_support::wendao::types::AttachmentKindArg;
use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct AttachmentsArgs {
    /// Optional query term (filename/path/source fields).
    pub query: Option<String>,
    #[arg(short, long, default_value_t = 50)]
    pub limit: usize,
    /// Attachment extension filter (repeatable; with/without leading dot).
    #[arg(long = "ext", value_name = "EXT", num_args = 1..)]
    pub exts: Vec<String>,
    /// Attachment kind filter (repeatable).
    #[arg(long = "kind", value_enum, num_args = 1..)]
    pub kinds: Vec<AttachmentKindArg>,
    #[arg(long, default_value_t = false)]
    pub case_sensitive: bool,
}
