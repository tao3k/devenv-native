//! Telegram webhook handler branch for request entry and response mapping.

mod entry;
mod ingest;

pub(in crate::channels::telegram::runtime::webhook) use entry::telegram_webhook_handler;
