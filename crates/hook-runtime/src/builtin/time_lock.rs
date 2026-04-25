//! TimeLock — block liquidity actions until an unlock timestamp.
//!
//! Category: timing. Callbacks: beforeAddLiquidity, beforeRemoveLiquidity.
//! Cable: yellow.
//!
//! Vetoes add/remove-liquidity while `ctx.timestamp < unlock_ts`. Models the
//! common vesting / cliff pattern where LPs commit capital for a fixed window.

use crate::builtin::HookCategory;
use crate::registry::Hook;
use crate::{HookCallback, HookContext, HookResult};

#[derive(Clone, Copy, Debug, Default)]
pub struct TimeLock {
    /// Unix timestamp before which liquidity actions are rejected.
    pub unlock_ts: i64,
}

impl TimeLock {
    pub const SLUG: &'static str = "time-lock";
    pub const CATEGORY: HookCategory = HookCategory::Timing;

    pub fn new(unlock_ts: i64) -> Self {
        TimeLock { unlock_ts }
    }

    pub fn is_unlocked(&self, now: i64) -> bool {
        now >= self.unlock_ts
    }
}

impl Hook for TimeLock {
    fn slug(&self) -> &'static str {
        Self::SLUG
    }

    fn callbacks(&self) -> &'static [HookCallback] {
        &[
            HookCallback::BeforeAddLiquidity,
            HookCallback::BeforeRemoveLiquidity,
        ]
    }

    fn evaluate(&self, ctx: &HookContext) -> HookResult {
        if self.is_unlocked(ctx.timestamp) {
            HookResult::allow()
        } else {
            HookResult::deny("time-lock: liquidity is locked until unlock_ts")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Dex;

    fn lp_ctx(now: i64) -> HookContext {
        let mut c = HookContext::new(
            HookCallback::BeforeRemoveLiquidity,
            Dex::RaydiumClmm,
            [3u8; 32],
        );
        c.timestamp = now;
        c
    }

    #[test]
    fn locked_before_unlock() {
        let h = TimeLock::new(1_000);
        let r = h.evaluate(&lp_ctx(999));
        assert!(!r.allow);
    }

    #[test]
    fn unlocked_at_and_after() {
        let h = TimeLock::new(1_000);
        assert!(h.evaluate(&lp_ctx(1_000)).allow);
        assert!(h.evaluate(&lp_ctx(5_000)).allow);
    }
}
