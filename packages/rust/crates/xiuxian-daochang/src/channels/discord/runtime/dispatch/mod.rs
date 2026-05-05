//! Discord dispatch coordinates message generation, command routing, session policy, and foreground send paths.

mod generation;
mod handler;
mod preview;
mod stop;
mod support;
mod turn;

pub(in crate::channels::discord::runtime) use handler::{
    process_discord_message, process_discord_message_with_interrupt,
    test_interrupted_reply_is_suppressed,
};
