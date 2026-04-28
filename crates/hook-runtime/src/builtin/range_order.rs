//! RangeOrder — one-sided order that fills as price crosses a target tick.
//!
//! Category: range. Callbacks: afterSwap. Cable: cyan.
//!
//! After each swap the hook checks whether the pool tick has crossed
//! `tick_target` in the configured direction. When crossed it reports a fill
//! (allow + reason); otherwise it is a no-op. Mirrors the Uniswap v4 range-order
//! pattern where limit-style orders are expressed as concentrated single-sided
//! liquidity that converts at a tick boundary.

use crate::builtin::HookCategory;
use crate::registry::Hook;
use crate::{HookCallback, HookContext, HookResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    /// Fills when the tick rises to/above the target.
    #[default]
    Above,
    /// Fills when the tick falls to/below the target.
    Below,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RangeOrder {
    pub tick_target: i32,
    pub direction: Direction,
}

impl RangeOrder {
    pub const SLUG: &'static str = "range-order";
    pub const CATEGORY: HookCategory = HookCategory::Range;

    pub fn new(tick_target: i32, direction: Direction) -> Self {
        RangeOrder {
            tick_target,
            direction,
        }
    }

    pub fn is_filled(&self, tick: i32) -> bool {
        match self.direction {
            Direction::Above => tick >= self.tick_target,
            Direction::Below => tick <= self.tick_target,
        }
    }
}

impl Hook for RangeOrder {
    fn slug(&self) -> &'static str {
        Self::SLUG
    }

    fn callbacks(&self) -> &'static [HookCallback] {
        &[HookCallback::AfterSwap]
    }

    fn evaluate(&self, ctx: &HookContext) -> HookResult {
        // Range orders never veto a swap; they observe the post-swap tick.
        let mut r = HookResult::allow();
        if self.is_filled(ctx.tick) {
            r.reason = Some("range-order: target tick crossed, order filled");
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Dex;

    fn after_swap_at(tick: i32) -> HookContext {
        let mut c = HookContext::new(HookCallback::AfterSwap, Dex::OrcaWhirlpool, [4u8; 32]);
        c.tick = tick;
        c
    }

    #[test]
    fn fills_above() {
        let h = RangeOrder::new(100, Direction::Above);
        assert!(h.is_filled(100));
        assert!(h.is_filled(150));
        assert!(!h.is_filled(99));
    }

    #[test]
    fn fills_below() {
        let h = RangeOrder::new(100, Direction::Below);
        assert!(h.is_filled(100));
        assert!(h.is_filled(50));
        assert!(!h.is_filled(101));
    }

    #[test]
    fn evaluate_never_vetoes() {
        let h = RangeOrder::new(100, Direction::Above);
        assert!(h.evaluate(&after_swap_at(50)).allow);
        assert!(h.evaluate(&after_swap_at(200)).allow);
    }
}
