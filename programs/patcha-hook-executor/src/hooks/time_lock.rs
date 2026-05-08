//! TimeLock — block liquidity actions until an unlock timestamp.
//! Category: timing. Callbacks: beforeAddLiquidity, beforeRemoveLiquidity.
//! Cable: yellow.
//!
//! Port of `crates/hook-runtime/src/builtin/time_lock.rs`. Vetoes add/remove
//! liquidity while `ctx.timestamp < unlock_ts`.

use super::{reason, HookOutcome, TriggerCtx};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct TimeLockParams {
    /// Unix timestamp before which liquidity actions are rejected.
    pub unlock_ts: i64,
}

pub fn is_unlocked(p: &TimeLockParams, now: i64) -> bool {
    now >= p.unlock_ts
}

pub fn evaluate(ctx: &TriggerCtx, blob: &[u8]) -> Result<HookOutcome> {
    let p = super::decode_params::<TimeLockParams>(blob)?;
    if is_unlocked(&p, ctx.timestamp) {
        Ok(HookOutcome::allow())
    } else {
        Ok(HookOutcome::deny(reason::TIME_LOCK_LOCKED))
    }
}
