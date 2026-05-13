//! Consensus voting branch modules split by submit, payload, and quorum behavior.

#[path = "consensus/manager/voting/submit.rs"]
mod submit;
#[path = "consensus/manager/voting/timeout.rs"]
mod timeout;
#[path = "consensus/manager/voting/winner.rs"]
mod winner;
