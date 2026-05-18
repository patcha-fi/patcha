//! Raydium CLMM adapter — maps a Raydium concentrated-liquidity lifecycle event
//! into a neutral [`TriggerCtx`].
//!
//! Symmetric with the Orca adapter: the integrator supplies the pool, signer,
//! amount, and current tick; this module stamps the venue tag and timestamp.

use crate::hooks::TriggerCtx;
use crate::state::DEX_RAYDIUM;
use anchor_lang::prelude::*;

/// Raydium CLMM program id (mainnet), kept for the integration boundary.
pub const RAYDIUM_CLMM_PROGRAM_ID: Pubkey =
    pubkey!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

/// Build a [`TriggerCtx`] for a Raydium CLMM swap event.
pub fn map_swap(
    callback: u8,
    pool: Pubkey,
    sender: Pubkey,
    amount_in: u64,
    tick: i32,
    timestamp: i64,
) -> TriggerCtx {
    let _ = pool;
    TriggerCtx {
        callback,
        dex: DEX_RAYDIUM,
        sender,
        amount_in,
        tick,
        timestamp,
    }
}
