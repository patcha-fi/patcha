//! Orca Whirlpools adapter — maps a whirlpool lifecycle event into a neutral
//! [`TriggerCtx`].
//!
//! The integrator supplies the whirlpool account, the initiating signer, and the
//! swap amount / current tick read from the whirlpool state; this module stamps
//! the venue tag and the slot timestamp so the builtin hooks evaluate the same
//! context regardless of venue.

use crate::hooks::TriggerCtx;
use crate::state::DEX_ORCA;
use anchor_lang::prelude::*;

/// Orca Whirlpools program id (mainnet), kept for the integration boundary.
pub const WHIRLPOOL_PROGRAM_ID: Pubkey = pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

/// Build a [`TriggerCtx`] for a whirlpool swap event.
pub fn map_swap(
    callback: u8,
    pool: Pubkey,
    sender: Pubkey,
    amount_in: u64,
    tick: i32,
    timestamp: i64,
) -> TriggerCtx {
    // `pool` is carried by the installation account; kept in the signature so
    // the mapping reads naturally at the call site.
    let _ = pool;
    TriggerCtx {
        callback,
        dex: DEX_ORCA,
        sender,
        amount_in,
        tick,
        timestamp,
    }
}
