//! RangeOrder — one-sided order that fills as price crosses a target tick.
//! Category: range. Callbacks: afterSwap. Cable: cyan.
//!
//! Port of `crates/hook-runtime/src/builtin/range_order.rs`. Never vetoes a
//! swap; observes the post-swap tick and reports a fill via the reason code.

use super::{reason, HookOutcome, TriggerCtx};
use anchor_lang::prelude::*;

/// Fill direction. 0 = Above (fills when tick rises to/above target),
/// 1 = Below (fills when tick falls to/below target). Mirrors `Direction` in
/// the runtime (default = Above).
pub const DIR_ABOVE: u8 = 0;
pub const DIR_BELOW: u8 = 1;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct RangeOrderParams {
    pub tick_target: i32,
    /// `DIR_ABOVE` (default) or `DIR_BELOW`.
    pub direction: u8,
}

pub fn is_filled(p: &RangeOrderParams, tick: i32) -> bool {
    match p.direction {
        DIR_BELOW => tick <= p.tick_target,
        // DIR_ABOVE and any other value default to "above" (runtime default).
        _ => tick >= p.tick_target,
    }
}

pub fn evaluate(ctx: &TriggerCtx, blob: &[u8]) -> Result<HookOutcome> {
    let p = super::decode_params::<RangeOrderParams>(blob)?;
    let mut outcome = HookOutcome::allow();
    if is_filled(&p, ctx.tick) {
        outcome = outcome.with_reason(reason::RANGE_ORDER_FILLED);
    }
    Ok(outcome)
}
