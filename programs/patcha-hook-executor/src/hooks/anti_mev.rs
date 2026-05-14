//! AntiMEV — dampen sandwich/MEV extraction with a per-swap price-move cap.
//! Category: mev. Callbacks: beforeSwap, afterSwap. Cable: purple.
//!
//! Port of `crates/hook-runtime/src/builtin/anti_mev.rs`. On `beforeSwap` the
//! hook estimates the price impact of the incoming swap (linear depth model) and
//! vetoes swaps that would move price beyond `max_price_move_bps`; allowed swaps
//! are credited with the headroom as `mev_bps_saved`.

use super::{reason, HookOutcome, TriggerCtx, CB_BEFORE_SWAP};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct AntiMevParams {
    /// Reject swaps whose estimated per-block price move exceeds this.
    pub max_price_move_bps: u32,
    /// Reference liquidity depth (base lamports) used to estimate impact.
    pub reference_depth: u64,
}

impl Default for AntiMevParams {
    fn default() -> Self {
        AntiMevParams {
            max_price_move_bps: 50,
            reference_depth: 1_000_000_000, // 1 SOL-equivalent
        }
    }
}

/// Estimated price impact of a swap in basis points (linear depth model).
pub fn estimated_move_bps(p: &AntiMevParams, amount_in: u64) -> u32 {
    if p.reference_depth == 0 {
        return 0;
    }
    let bps = (amount_in as u128 * 10_000) / p.reference_depth as u128;
    bps.min(u32::MAX as u128) as u32
}

pub fn evaluate(ctx: &TriggerCtx, blob: &[u8]) -> Result<HookOutcome> {
    let p = super::decode_params::<AntiMevParams>(blob)?;
    if ctx.callback != CB_BEFORE_SWAP {
        return Ok(HookOutcome::allow());
    }
    let move_bps = estimated_move_bps(&p, ctx.amount_in);
    if move_bps > p.max_price_move_bps {
        Ok(HookOutcome::deny(reason::ANTI_MEV_BLOCKED))
    } else {
        // Credit the headroom we enforced as MEV the attacker couldn't take.
        Ok(HookOutcome::allow()
            .with_mev_saved(p.max_price_move_bps - move_bps)
            .with_reason(reason::ANTI_MEV_CREDITED))
    }
}
