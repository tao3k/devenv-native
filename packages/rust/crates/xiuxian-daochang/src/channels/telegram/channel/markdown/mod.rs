//! Telegram markdown branch for escaping and parse-mode formatting.

mod escape;
mod html;
mod markdown_v2;
mod options;

pub use html::markdown_to_telegram_html;
pub use markdown_v2::markdown_to_telegram_markdown_v2;
